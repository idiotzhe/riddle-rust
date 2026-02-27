@echo off
setlocal enabledelayedexpansion

set ROOT_DIR=%~dp0
cd /d "%ROOT_DIR%"

echo [1/4] Building Vue Admin Frontend...
cd admin
call bun run build -- --outDir ../template/admin --emptyOutDir
if %errorlevel% neq 0 (
    echo Error: Vue build failed.
    pause
    exit /b %errorlevel%
)
cd /d "%ROOT_DIR%"

echo [2/4] Preparing src-tauri...
if not exist "src-tauri\src\handlers" (
    mkdir "src-tauri\src\handlers"
)
copy "backend-rust\src\models.rs" "src-tauri\src\" /Y
copy "backend-rust\src\utils.rs" "src-tauri\src\" /Y
copy "backend-rust\src\db.rs" "src-tauri\src\" /Y
xcopy "backend-rust\src\handlers" "src-tauri\src\handlers" /E /I /Y /Q

echo [3/4] Building Tauri Desktop App (Standalone EXE)...
cd /d "%ROOT_DIR%"
:: 使用 bun 运行 tauri build
call bun tauri build
if %errorlevel% neq 0 (
    echo Error: Tauri build failed.
    pause
    exit /b %errorlevel%
)

echo [4/4] Copying final binary...
cd /d "%ROOT_DIR%"
if not exist "dist-desktop" mkdir "dist-desktop"
copy "src-tauri\target\release\lantern-riddle.exe" "dist-desktop\lantern-riddle-admin.exe" /Y
if exist "lantern.db" (
    copy "lantern.db" "dist-desktop\lantern.db" /Y
)

echo.
echo ======================================================
echo Desktop App Build Success!
echo.
echo The 'dist-desktop\lantern-riddle-admin.exe' is a 
echo standalone Windows application.
echo ======================================================

pause
