@echo off
echo ========================================
echo   Lantern Riddle Project Build Script (Rust Version)
echo ========================================

echo.
echo [1/2] Building Admin Vue Frontend...
cd admin
call bun install
call bun run build -- --outDir ../template/admin --emptyOutDir
if %errorlevel% neq 0 (
    echo Frontend build failed!
    pause
    exit /b %errorlevel%
)
cd ..

echo.
echo [2/2] Building Tauri App (with integrated Rust backend)...
call bun tauri build
if %errorlevel% neq 0 (
    echo Tauri build failed!
    pause
    exit /b %errorlevel%
)

echo.
echo ========================================
echo   Build Successful! 
echo   EXE location: src-tauri\target\release\lantern-riddle.exe
echo ========================================
pause
