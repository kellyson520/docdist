# DocDist 卸载清理脚本

## Windows
```powershell
# 以管理员身份运行 PowerShell
# 删除 DocDist 数据目录
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\docdist" -ErrorAction SilentlyContinue
Write-Host "DocDist 数据已清理: $env:LOCALAPPDATA\docdist"

# 删除 DocDist 配置
Remove-Item -Recurse -Force "$env:APPDATA\docdist" -ErrorAction SilentlyContinue
Write-Host "DocDist 配置已清理: $env:APPDATA\docdist"
```

## macOS
```bash
# 删除 DocDist 数据目录
rm -rf ~/Library/Application\ Support/docdist
echo "DocDist 数据已清理"

# 删除 DocDist 偏好设置
rm -rf ~/Library/Preferences/com.docdist.app.plist 2>/dev/null
echo "DocDist 配置已清理"
```

## Linux
```bash
# 删除 DocDist 数据目录
rm -rf ~/.local/share/docdist
echo "DocDist 数据已清理"

# 删除 DocDist 配置
rm -rf ~/.config/docdist 2>/dev/null
echo "DocDist 配置已清理"
```

## 说明

DocDist 的数据存储在以下位置：

| 平台 | 数据目录 | 内容 |
|------|---------|------|
| Windows | `%LOCALAPPDATA%\docdist` | data.db, config.json, chunks/, logs/ |
| macOS | `~/Library/Application Support/docdist` | 同上 |
| Linux | `~/.local/share/docdist` | 同上 |

卸载 DocDist 后，这些数据**不会自动删除**。运行对应平台的清理脚本可完全移除所有数据。
