@echo off
cd /d "%~dp0"
echo 启动图片归档工具...
echo 请确保已运行 npm install
npm run tauri dev
pause
