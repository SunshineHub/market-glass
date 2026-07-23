use std::{path::Path, str::FromStr, sync::Mutex};

use async_trait::async_trait;
use market_glass_application::{PortfolioRepository, RepositoryError};
use market_glass_domain::{AssetKind, Position};
use rusqlite::{Connection, OptionalExtension, params};
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct SqlitePortfolioRepository {
    connection: Mutex<Connection>,
}

impl SqlitePortfolioRepository {
    pub fn open(path: &Path) -> Result<Self, RepositoryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(repository_error)?;
        }
        let connection = Connection::open(path).map_err(repository_error)?;
        connection
            .execute_batch(include_str!("../migrations/0001_initial.sql"))
            .map_err(repository_error)?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, RepositoryError> {
        self.connection
            .lock()
            .map_err(|_| RepositoryError::Unavailable("SQLite lock poisoned".into()))
    }
}

#[async_trait]
impl PortfolioRepository for SqlitePortfolioRepository {
    async fn list_positions(&self) -> Result<Vec<Position>, RepositoryError> {
        let connection = self.connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, kind, code, name, units, total_cost, manual_value, \
                 manual_day_percent, provider, strategy FROM positions ORDER BY rowid",
            )
            .map_err(repository_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(StoredPosition {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    code: row.get(2)?,
                    name: row.get(3)?,
                    units: row.get(4)?,
                    total_cost: row.get(5)?,
                    manual_value: row.get(6)?,
                    manual_day_percent: row.get(7)?,
                    provider: row.get(8)?,
                    strategy: row.get(9)?,
                })
            })
            .map_err(repository_error)?;

        rows.map(|row| row.map_err(repository_error).and_then(Position::try_from))
            .collect()
    }

    async fn add_position(&self, position: &Position) -> Result<(), RepositoryError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction().map_err(repository_error)?;
        let merged = if position.kind == AssetKind::Fund {
            position
                .code
                .as_deref()
                .map(|code| find_fund_by_code(&transaction, code))
                .transpose()?
                .flatten()
                .map(|existing| merge_fund(existing, position))
                .unwrap_or_else(|| position.clone())
        } else {
            position.clone()
        };
        persist_position(&transaction, &merged)?;
        transaction.commit().map_err(repository_error)
    }

    async fn add_positions(&self, positions: &[Position]) -> Result<(), RepositoryError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction().map_err(repository_error)?;
        for position in positions {
            let merged = if position.kind == AssetKind::Fund {
                position
                    .code
                    .as_deref()
                    .map(|code| find_fund_by_code(&transaction, code))
                    .transpose()?
                    .flatten()
                    .map(|existing| merge_fund(existing, position))
                    .unwrap_or_else(|| position.clone())
            } else {
                position.clone()
            };
            persist_position(&transaction, &merged)?;
        }
        transaction.commit().map_err(repository_error)
    }

    async fn delete_positions(&self, ids: &[Uuid]) -> Result<(), RepositoryError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction().map_err(repository_error)?;
        for id in ids {
            transaction
                .execute("DELETE FROM positions WHERE id = ?1", [id.to_string()])
                .map_err(repository_error)?;
        }
        transaction.commit().map_err(repository_error)
    }

    async fn upsert_position(&self, position: &Position) -> Result<(), RepositoryError> {
        let connection = self.connection()?;
        persist_position(&connection, position)
    }

    async fn upsert_positions(&self, positions: &[Position]) -> Result<(), RepositoryError> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction().map_err(repository_error)?;
        for position in positions {
            persist_position(&transaction, position)?;
        }
        transaction.commit().map_err(repository_error)
    }

    async fn get_string_setting(
        &self,
        key: &str,
        default: &str,
    ) -> Result<String, RepositoryError> {
        let connection = self.connection()?;
        connection
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get::<_, String>(0)
            })
            .optional()
            .map_err(repository_error)
            .map(|value| value.unwrap_or_else(|| default.to_owned()))
    }

    async fn set_string_setting(&self, key: &str, value: &str) -> Result<(), RepositoryError> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO settings (key, value, updated_at)
                 VALUES (?1, ?2, CURRENT_TIMESTAMP)
                 ON CONFLICT(key) DO UPDATE SET
                    value = excluded.value,
                    updated_at = CURRENT_TIMESTAMP",
                params![key, value],
            )
            .map_err(repository_error)?;
        Ok(())
    }

    async fn get_bool_setting(&self, key: &str, default: bool) -> Result<bool, RepositoryError> {
        let connection = self.connection()?;
        let value = connection
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get::<_, String>(0)
            })
            .optional()
            .map_err(repository_error)?;
        Ok(value.map(|value| value == "true").unwrap_or(default))
    }

    async fn set_bool_setting(&self, key: &str, value: bool) -> Result<(), RepositoryError> {
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO settings (key, value, updated_at)
                 VALUES (?1, ?2, CURRENT_TIMESTAMP)
                 ON CONFLICT(key) DO UPDATE SET
                    value = excluded.value,
                    updated_at = CURRENT_TIMESTAMP",
                params![key, value.to_string()],
            )
            .map_err(repository_error)?;
        Ok(())
    }
}

fn persist_position(connection: &Connection, position: &Position) -> Result<(), RepositoryError> {
    let existing_id = if position.kind == AssetKind::Fund {
        position
            .code
            .as_deref()
            .map(|code| {
                connection
                    .query_row(
                        "SELECT id FROM positions WHERE kind = 'fund' AND code = ?1",
                        [code],
                        |row| row.get::<_, String>(0),
                    )
                    .optional()
                    .map_err(repository_error)
            })
            .transpose()?
            .flatten()
    } else {
        None
    };
    let id = existing_id.unwrap_or_else(|| position.id.to_string());

    connection
        .execute(
            "INSERT INTO positions (
                id, kind, code, name, units, total_cost, manual_value,
                manual_day_percent, provider, strategy, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET
                kind = excluded.kind,
                code = excluded.code,
                name = excluded.name,
                units = excluded.units,
                total_cost = excluded.total_cost,
                manual_value = excluded.manual_value,
                manual_day_percent = excluded.manual_day_percent,
                provider = excluded.provider,
                strategy = excluded.strategy,
                updated_at = CURRENT_TIMESTAMP",
            params![
                id,
                kind_to_string(position.kind),
                position.code,
                position.name,
                position.units.to_string(),
                position.total_cost.to_string(),
                position.manual_value.map(|value| value.to_string()),
                position.manual_day_percent.map(|value| value.to_string()),
                position.provider,
                position.strategy,
            ],
        )
        .map_err(repository_error)?;
    Ok(())
}

fn find_fund_by_code(
    connection: &Connection,
    code: &str,
) -> Result<Option<Position>, RepositoryError> {
    connection
        .query_row(
            "SELECT id, kind, code, name, units, total_cost, manual_value,
                    manual_day_percent, provider, strategy
             FROM positions WHERE kind = 'fund' AND code = ?1",
            [code],
            |row| {
                Ok(StoredPosition {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    code: row.get(2)?,
                    name: row.get(3)?,
                    units: row.get(4)?,
                    total_cost: row.get(5)?,
                    manual_value: row.get(6)?,
                    manual_day_percent: row.get(7)?,
                    provider: row.get(8)?,
                    strategy: row.get(9)?,
                })
            },
        )
        .optional()
        .map_err(repository_error)?
        .map(Position::try_from)
        .transpose()
}

fn merge_fund(existing: Position, incoming: &Position) -> Position {
    Position {
        id: existing.id,
        kind: AssetKind::Fund,
        code: incoming.code.clone(),
        name: incoming.name.clone(),
        units: existing.units + incoming.units,
        total_cost: existing.total_cost + incoming.total_cost,
        manual_value: incoming.manual_value.or(existing.manual_value),
        manual_day_percent: incoming.manual_day_percent.or(existing.manual_day_percent),
        provider: incoming.provider.clone(),
        strategy: if incoming.strategy.trim().is_empty() || incoming.strategy == "未分类" {
            existing.strategy
        } else {
            incoming.strategy.clone()
        },
    }
}

struct StoredPosition {
    id: String,
    kind: String,
    code: Option<String>,
    name: String,
    units: String,
    total_cost: String,
    manual_value: Option<String>,
    manual_day_percent: Option<String>,
    provider: String,
    strategy: String,
}

impl TryFrom<StoredPosition> for Position {
    type Error = RepositoryError;

    fn try_from(value: StoredPosition) -> Result<Self, Self::Error> {
        Ok(Position {
            id: Uuid::parse_str(&value.id).map_err(invalid_data)?,
            kind: parse_kind(&value.kind)?,
            code: value.code,
            name: value.name,
            units: Decimal::from_str(&value.units).map_err(invalid_data)?,
            total_cost: Decimal::from_str(&value.total_cost).map_err(invalid_data)?,
            manual_value: value
                .manual_value
                .map(|item| Decimal::from_str(&item).map_err(invalid_data))
                .transpose()?,
            manual_day_percent: value
                .manual_day_percent
                .map(|item| Decimal::from_str(&item).map_err(invalid_data))
                .transpose()?,
            provider: value.provider,
            strategy: value.strategy,
        })
    }
}

fn kind_to_string(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Fund => "fund",
        AssetKind::Advisory => "advisory",
        AssetKind::Cash => "cash",
    }
}

fn parse_kind(kind: &str) -> Result<AssetKind, RepositoryError> {
    match kind {
        "fund" => Ok(AssetKind::Fund),
        "advisory" => Ok(AssetKind::Advisory),
        "cash" => Ok(AssetKind::Cash),
        other => Err(RepositoryError::InvalidData(format!(
            "unknown asset kind: {other}"
        ))),
    }
}

fn repository_error(error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::Unavailable(error.to_string())
}

fn invalid_data(error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::InvalidData(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn persists_positions_and_privacy_mode() {
        let repository = SqlitePortfolioRepository::open(Path::new(":memory:")).unwrap();
        let position = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Advisory,
            code: None,
            name: "稳健投顾".into(),
            units: Decimal::ZERO,
            total_cost: Decimal::new(10_000, 0),
            manual_value: Some(Decimal::new(10_200, 0)),
            manual_day_percent: Some(Decimal::new(15, 2)),
            provider: "manual".into(),
            strategy: "balanced".into(),
        };

        repository.upsert_position(&position).await.unwrap();
        repository
            .set_bool_setting("privacy_mode", true)
            .await
            .unwrap();

        let positions = repository.list_positions().await.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].name, "稳健投顾");
        assert!(
            repository
                .get_bool_setting("privacy_mode", false)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn batch_import_updates_a_fund_with_the_same_code_atomically() {
        let repository = SqlitePortfolioRepository::open(Path::new(":memory:")).unwrap();
        let first = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Fund,
            code: Some("005827".into()),
            name: "第一版名称".into(),
            units: Decimal::new(100, 0),
            total_cost: Decimal::new(100, 0),
            manual_value: None,
            manual_day_percent: None,
            provider: "import".into(),
            strategy: "默认分组".into(),
        };
        let mut updated = first.clone();
        updated.id = Uuid::new_v4();
        updated.name = "第二版名称".into();
        updated.units = Decimal::new(200, 0);

        repository
            .upsert_positions(&[first, updated])
            .await
            .unwrap();

        let positions = repository.list_positions().await.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].name, "第二版名称");
        assert_eq!(positions[0].units, Decimal::new(200, 0));
    }

    #[tokio::test]
    async fn adding_an_existing_fund_accumulates_units_and_cost() {
        let repository = SqlitePortfolioRepository::open(Path::new(":memory:")).unwrap();
        let first = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Fund,
            code: Some("001618".into()),
            name: "科技基金".into(),
            units: Decimal::new(1255, 1),
            total_cost: Decimal::new(20050, 2),
            manual_value: None,
            manual_day_percent: None,
            provider: "manual".into(),
            strategy: "科技".into(),
        };
        let mut additional = first.clone();
        additional.id = Uuid::new_v4();
        additional.units = Decimal::new(745, 1);
        additional.total_cost = Decimal::new(9950, 2);

        repository.add_position(&first).await.unwrap();
        repository.add_position(&additional).await.unwrap();

        let positions = repository.list_positions().await.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].units, Decimal::new(200, 0));
        assert_eq!(positions[0].total_cost, Decimal::new(300, 0));
    }

    #[tokio::test]
    async fn adding_a_batch_also_accumulates_existing_funds() {
        let repository = SqlitePortfolioRepository::open(Path::new(":memory:")).unwrap();
        let first = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Fund,
            code: Some("005827".into()),
            name: "蓝筹基金".into(),
            units: Decimal::new(100, 0),
            total_cost: Decimal::new(200, 0),
            manual_value: None,
            manual_day_percent: None,
            provider: "import".into(),
            strategy: "综合配置".into(),
        };
        let mut additional = first.clone();
        additional.id = Uuid::new_v4();
        additional.units = Decimal::new(50, 0);
        additional.total_cost = Decimal::new(80, 0);

        repository.add_position(&first).await.unwrap();
        repository.add_positions(&[additional]).await.unwrap();

        let positions = repository.list_positions().await.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].units, Decimal::new(150, 0));
        assert_eq!(positions[0].total_cost, Decimal::new(280, 0));
    }

    #[tokio::test]
    async fn deleting_selected_positions_is_atomic_and_scoped() {
        let repository = SqlitePortfolioRepository::open(Path::new(":memory:")).unwrap();
        let first = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Fund,
            code: Some("001618".into()),
            name: "基金一".into(),
            units: Decimal::ZERO,
            total_cost: Decimal::ZERO,
            manual_value: None,
            manual_day_percent: None,
            provider: "manual".into(),
            strategy: "科技".into(),
        };
        let second = Position {
            id: Uuid::new_v4(),
            kind: AssetKind::Advisory,
            code: None,
            name: "投顾二".into(),
            units: Decimal::ZERO,
            total_cost: Decimal::ZERO,
            manual_value: Some(Decimal::ZERO),
            manual_day_percent: Some(Decimal::ZERO),
            provider: "manual".into(),
            strategy: "稳健".into(),
        };
        repository.add_position(&first).await.unwrap();
        repository.add_position(&second).await.unwrap();

        repository.delete_positions(&[first.id]).await.unwrap();

        let positions = repository.list_positions().await.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].id, second.id);
    }
}
