@echo off
cd /d "%~dp0"
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo Build failed.
    exit /b %ERRORLEVEL%
)
echo.
echo Build complete: target\release\ludusavi.exe
