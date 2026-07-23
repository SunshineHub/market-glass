# Windows 自动打包与分发

项目通过 `.github/workflows/windows-installer.yml` 在 GitHub 托管的 Windows 构建机上生成 x64 NSIS 安装器。

## 第一次接入

1. 在 GitHub 新建一个空的私有仓库，不要勾选自动创建 README、`.gitignore` 或 License。
2. 在本地 `market-glass` 目录初始化 Git，并把当前源码推送到该仓库。
3. 打开仓库的 **Actions** 页面，选择 **Build Windows Installer**。
4. 点击 **Run workflow**，选择主分支后运行。
5. 构建成功后，在该次运行页面底部下载 `Market-Glass-Windows-x64-*` Artifact。

Artifact 是 GitHub 自动压缩的 ZIP，解压后包含：

- `Market Glass_*_x64-setup.exe`：给 Windows 10/11 x64 用户的安装器。
- `SHA256SUMS.txt`：用于核对安装器完整性。

## 版本发布

更新 `src-tauri/tauri.conf.json`、根目录 `package.json` 和工作区版本后提交代码，再创建以 `v` 开头的标签：

```bash
git tag v0.1.0
git push origin v0.1.0
```

标签会触发同一工作流，并创建一个 GitHub 草稿 Release。确认安装器和说明无误后，在 GitHub Release 页面点击发布，即可把下载链接发给其他人。

## 安全边界

- 当前工作流不会读取或打包开发者本机数据库，用户安装后会创建独立的本地数据目录。
- GitHub Artifact 默认保留 30 天；正式长期分发应使用 Release。
- 当前生成的安装器未做 Authenticode 签名，Windows SmartScreen 可能提示“未知发布者”。
- 面向公众发布前，应接入受信任的 Windows 代码签名服务或提交 Microsoft Store。
- 不要把证书、密码或私钥写入仓库；签名材料只能放在 GitHub Actions Secrets 或云签名服务中。
