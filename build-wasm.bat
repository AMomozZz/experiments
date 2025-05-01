@echo off
setlocal enabledelayedexpansion

cd guest-rs

echo Building the project...
cargo component build --target wasm32-wasip2 --release
if %errorlevel% neq 0 (
        echo Build failed.
        exit /b %errorlevel%
)

@REM cd ../guest-qs

@REM echo Building the project...
@REM cargo component build --target wasm32-wasip2 --release
@REM if %errorlevel% neq 0 (
@REM         echo Build failed.
@REM         exit /b %errorlevel%
@REM )