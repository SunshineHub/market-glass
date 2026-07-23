# 自动更新与签名发布

Market Glass 使用 Tauri Updater 和 GitHub Release 的静态 `latest.json` 清单完成 macOS arm64 与 Windows x64 应用内更新。

## 运行流程

1. 主窗口启动约 3 秒后静默检查 `releases/latest/download/latest.json`。
2. 有新版本时展示版本号、发布日期和更新说明；用户可以稍后处理或立即更新。
3. 应用下载与当前系统匹配的更新包并显示进度。
4. Tauri 使用内置公钥验证更新签名，校验失败时不会安装。
5. macOS 安装完成后重启应用；Windows 安装阶段会按安装器要求退出并自动完成更新。

用户的持仓数据库位于系统应用数据目录，安装包更新不会覆盖数据库。

## 密钥管理

- 公钥存放在 `src-tauri/tauri.conf.json`，可以公开。
- 私钥禁止提交到仓库；本机备份应存放在仓库外并限制文件权限。
- GitHub Actions 使用仓库 Secret `TAURI_SIGNING_PRIVATE_KEY`。
- 私钥丢失后，已经安装的客户端将无法验证后续更新，因此必须另行加密备份。

## 发布新版本

1. 同步修改 `Cargo.toml`、`package.json`、`apps/desktop-ui/package.json` 和 `src-tauri/tauri.conf.json` 中的版本号。
2. 完成 `pnpm typecheck`、`pnpm build` 和 `cargo test --workspace --locked`。
3. 推送代码后，在 GitHub Actions 运行 `Build and Release Installers`，或推送对应的 `v*` 标签。
4. 工作流会在 macOS 与 Windows runner 分别构建安装包、更新包和 `.sig` 文件。
5. 两个平台完成后，发布任务验证 `latest.json` 同时包含 `darwin-aarch64` 与 `windows-x86_64`，然后才把草稿 Release 正式发布。

本地生成签名 macOS 更新包：

```bash
TAURI_SIGNING_PRIVATE_KEY="/secure/path/market-glass-updater.key" \
TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \
pnpm tauri build --bundles app,dmg
```

不要在命令、日志、提交或 `.env` 文件中写入私钥正文。
