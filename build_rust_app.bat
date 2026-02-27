@echo off
setlocal enabledelayedexpansion

set ROOT_DIR=%~dp0
cd /d "%ROOT_DIR%"

echo [1/3] Cleaning up old dist...
if exist "dist" (
    rd /s /q "dist"
)
mkdir "dist"

echo [2/3] Compiling Standalone Rust binary...
cd /d "%ROOT_DIR%\backend-rust"
cargo build --release
if %errorlevel% neq 0 (
    echo Error: Rust compilation failed.
    pause
    exit /b %errorlevel%
)

echo [3/3] Copying final binary and database...
cd /d "%ROOT_DIR%"
copy "backend-rust\target\release\backend-rust.exe" "dist\lantern-riddle-standalone.exe" /Y
if exist "lantern.db" (
    copy "lantern.db" "dist\lantern.db" /Y
)

echo.
echo ======================================================
echo Standalone Build Success!
echo.
echo The 'dist\lantern-riddle-standalone.exe' now contains:
echo - All API Logic
echo - All HTML/JS/CSS Templates (Embedded)
echo - All Images and Assets (Embedded)
echo.
echo Note: 'lantern.db' is still required in the same folder
echo for data storage.
echo ======================================================

pause
