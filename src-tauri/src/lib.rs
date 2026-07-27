use std::sync::Arc;

use market_glass_application::{
    BootstrapPayloadDto, FundMetadata, OverviewService, OverviewSnapshotDto,
    PositionBatchUpdateResultDto, PositionInputDto,
};
use market_glass_infrastructure::SqlitePortfolioRepository;
use market_glass_providers::HybridMarketDataProvider;
use tauri::image::Image;
use tauri::menu::MenuBuilder;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

struct AppState {
    overview: Arc<OverviewService>,
}

fn tray_icon() -> Image<'static> {
    const SIZE: u32 = 44;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];

    let mut dot = |center_x: f32, center_y: f32, radius: f32| {
        let min_x = (center_x - radius).floor().max(0.0) as u32;
        let max_x = (center_x + radius).ceil().min((SIZE - 1) as f32) as u32;
        let min_y = (center_y - radius).floor().max(0.0) as u32;
        let max_y = (center_y + radius).ceil().min((SIZE - 1) as f32) as u32;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let distance =
                    ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                if distance <= radius {
                    let index = ((y * SIZE + x) * 4) as usize;
                    rgba[index + 3] = 255;
                }
            }
        }
    };

    let mut line = |from: (f32, f32), to: (f32, f32), radius: f32| {
        let steps = (to.0 - from.0).abs().max((to.1 - from.1).abs()).ceil() as u32;
        for step in 0..=steps.max(1) {
            let progress = step as f32 / steps.max(1) as f32;
            dot(
                from.0 + (to.0 - from.0) * progress,
                from.1 + (to.1 - from.1) * progress,
                radius,
            );
        }
    };

    line((6.0, 32.0), (18.0, 20.0), 2.1);
    line((18.0, 20.0), (26.0, 26.0), 2.1);
    line((26.0, 26.0), (38.0, 9.0), 2.1);
    line((7.0, 37.0), (38.0, 37.0), 1.4);
    dot(38.0, 9.0, 3.2);

    Image::new_owned(rgba, SIZE, SIZE)
}

#[tauri::command]
async fn get_bootstrap(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<BootstrapPayloadDto, String> {
    let mut payload = state
        .overview
        .bootstrap()
        .await
        .map_err(|error| error.to_string())?;
    update_tray_badge(window.app_handle(), &payload.overview)?;
    if window.label() == "mini" {
        payload.overview = snapshot_for_mini(payload.overview);
    }
    Ok(payload)
}

#[tauri::command]
async fn refresh_overview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<OverviewSnapshotDto, String> {
    let (snapshot, _) = state
        .overview
        .overview()
        .await
        .map_err(|error| error.to_string())?;
    emit_snapshot(&app, &snapshot)?;
    Ok(snapshot)
}

fn refresh_market_in_background(app: AppHandle, overview: Arc<OverviewService>) {
    tauri::async_runtime::spawn(async move {
        if let Ok((snapshot, _)) = overview.overview().await {
            let _ = emit_snapshot(&app, &snapshot);
        }
    });
}

#[tauri::command]
async fn lookup_fund(
    state: State<'_, AppState>,
    code: String,
) -> Result<Option<FundMetadata>, String> {
    state
        .overview
        .lookup_fund(code)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn set_privacy_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    state
        .overview
        .set_privacy_mode(enabled)
        .await
        .map_err(|error| error.to_string())?;
    app.emit("settings://privacy-changed", enabled)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn set_selected_indices(
    app: AppHandle,
    state: State<'_, AppState>,
    codes: Vec<String>,
) -> Result<Vec<String>, String> {
    let selected = state
        .overview
        .set_selected_indices(codes)
        .await
        .map_err(|error| error.to_string())?;
    app.emit("settings://indices-changed", &selected)
        .map_err(|error| error.to_string())?;
    Ok(selected)
}

#[tauri::command]
async fn set_market_indices(
    app: AppHandle,
    state: State<'_, AppState>,
    codes: Vec<String>,
) -> Result<Vec<String>, String> {
    let selected = state
        .overview
        .set_market_indices(codes)
        .await
        .map_err(|error| error.to_string())?;
    app.emit("settings://market-indices-changed", &selected)
        .map_err(|error| error.to_string())?;
    Ok(selected)
}

#[tauri::command]
async fn upsert_position(
    app: AppHandle,
    state: State<'_, AppState>,
    input: PositionInputDto,
) -> Result<OverviewSnapshotDto, String> {
    let snapshot = state
        .overview
        .upsert_position(input)
        .await
        .map_err(|error| error.to_string())?;
    emit_snapshot(&app, &snapshot)?;
    refresh_market_in_background(app, state.overview.clone());
    Ok(snapshot)
}

#[tauri::command]
async fn import_positions(
    app: AppHandle,
    state: State<'_, AppState>,
    inputs: Vec<PositionInputDto>,
) -> Result<OverviewSnapshotDto, String> {
    let snapshot = state
        .overview
        .import_positions(inputs)
        .await
        .map_err(|error| error.to_string())?;
    emit_snapshot(&app, &snapshot)?;
    refresh_market_in_background(app, state.overview.clone());
    Ok(snapshot)
}

#[tauri::command]
async fn update_positions_partial(
    app: AppHandle,
    state: State<'_, AppState>,
    inputs: Vec<PositionInputDto>,
) -> Result<PositionBatchUpdateResultDto, String> {
    let result = state
        .overview
        .update_positions_partial(inputs)
        .await
        .map_err(|error| error.to_string())?;
    emit_snapshot(&app, &result.snapshot)?;
    refresh_market_in_background(app, state.overview.clone());
    Ok(result)
}

#[tauri::command]
async fn delete_positions(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> Result<OverviewSnapshotDto, String> {
    let snapshot = state
        .overview
        .delete_positions(ids)
        .await
        .map_err(|error| error.to_string())?;
    emit_snapshot(&app, &snapshot)?;
    refresh_market_in_background(app, state.overview.clone());
    Ok(snapshot)
}

#[tauri::command]
fn minimize_current_window(window: WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|error| error.to_string())
}

#[tauri::command]
fn hide_current_window(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())
}

#[tauri::command]
fn show_window(app: AppHandle, kind: String) -> Result<(), String> {
    if !matches!(kind.as_str(), "main" | "mini") {
        return Err("unknown window kind".into());
    }
    open_window(&app, &kind)
}

#[tauri::command]
fn export_json(app: AppHandle, filename: String, content: String) -> Result<String, String> {
    if filename.is_empty()
        || filename.len() > 120
        || !filename.ends_with(".json")
        || filename.contains(['/', '\\'])
    {
        return Err("invalid export filename".into());
    }
    if content.len() > 10 * 1024 * 1024 {
        return Err("export content is too large".into());
    }

    let directory = app
        .path()
        .download_dir()
        .map_err(|error| error.to_string())?;
    let stem = filename.trim_end_matches(".json");
    let mut path = directory.join(&filename);
    let mut suffix = 2_u16;
    while path.exists() {
        path = directory.join(format!("{stem}-{suffix}.json"));
        suffix = suffix.saturating_add(1);
    }
    std::fs::write(&path, content).map_err(|error| error.to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

fn open_window(app: &AppHandle, kind: &str) -> Result<(), String> {
    let window = app
        .get_webview_window(kind)
        .ok_or_else(|| format!("window not found: {kind}"))?;
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

fn snapshot_for_mini(mut snapshot: OverviewSnapshotDto) -> OverviewSnapshotDto {
    snapshot.assets.retain(|asset| asset.kind == "fund");
    snapshot.allocation.clear();
    snapshot.asset_trend.clear();
    snapshot
}

fn emit_snapshot(app: &AppHandle, snapshot: &OverviewSnapshotDto) -> Result<(), String> {
    update_tray_badge(app, snapshot)?;
    app.emit_to("main", "portfolio://main-snapshot-updated", snapshot)
        .map_err(|error| error.to_string())?;
    app.emit_to(
        "mini",
        "portfolio://mini-snapshot-updated",
        snapshot_for_mini(snapshot.clone()),
    )
    .map_err(|error| error.to_string())
}

fn tray_percent_label(value: f64) -> String {
    let value = if value.abs() < 0.005 { 0.0 } else { value };
    if value > 0.0 {
        format!("+{value:.2}%")
    } else if value < 0.0 {
        format!("−{:.2}%", value.abs())
    } else {
        "0.00%".into()
    }
}

fn update_tray_badge(app: &AppHandle, snapshot: &OverviewSnapshotDto) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(tray) = app.tray_by_id("market-glass-tray") {
            let label = tray_percent_label(snapshot.day_profit_percent);
            tray.set_title(Some(&label))
                .map_err(|error| error.to_string())?;
            tray.set_tooltip(Some(format!("Market Glass · 今日 {label}")))
                .map_err(|error| error.to_string())?;
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, snapshot);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let repository = Arc::new(
                SqlitePortfolioRepository::open(&data_dir.join("market-glass.sqlite3"))
                    .map_err(|error| error.to_string())?,
            );
            let market =
                Arc::new(HybridMarketDataProvider::new().map_err(|error| error.to_string())?);
            let overview = Arc::new(OverviewService::new(repository, market));
            app.manage(AppState {
                overview: overview.clone(),
            });

            for label in ["main", "mini"] {
                if let Some(window) = app.get_webview_window(label) {
                    let window_to_hide = window.clone();
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            api.prevent_close();
                            let _ = window_to_hide.hide();
                        }
                    });
                }
            }

            let tray_menu = MenuBuilder::new(app)
                .text("show-main", "打开主窗口")
                .text("show-mini", "显示极简窗口")
                .separator()
                .text("refresh", "立即刷新")
                .separator()
                .text("quit", "退出 Market Glass")
                .build()?;
            let overview_for_tray = overview.clone();
            TrayIconBuilder::with_id("market-glass-tray")
                .icon(tray_icon())
                .icon_as_template(true)
                .title("—")
                .tooltip("Market Glass")
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if matches!(
                        event,
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                    ) {
                        let _ = open_window(tray.app_handle(), "mini");
                    }
                })
                .on_menu_event(move |app_handle, event| match event.id().as_ref() {
                    "show-main" => {
                        let _ = open_window(app_handle, "main");
                    }
                    "show-mini" => {
                        let _ = open_window(app_handle, "mini");
                    }
                    "refresh" => {
                        let service = overview_for_tray.clone();
                        let app = app_handle.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Ok((snapshot, _)) = service.overview().await {
                                let _ = emit_snapshot(&app, &snapshot);
                            }
                        });
                    }
                    "quit" => app_handle.exit(0),
                    _ => {}
                })
                .build(app)?;

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
                interval.tick().await;
                loop {
                    interval.tick().await;
                    if let Ok((snapshot, _)) = overview.overview().await {
                        let _ = emit_snapshot(&app_handle, &snapshot);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_bootstrap,
            refresh_overview,
            lookup_fund,
            set_privacy_mode,
            set_selected_indices,
            set_market_indices,
            upsert_position,
            import_positions,
            update_positions_partial,
            delete_positions,
            export_json,
            minimize_current_window,
            hide_current_window,
            show_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running Market Glass");
}
