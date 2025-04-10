@echo off
setlocal enabledelayedexpansion

cd guest-rs

echo Building the project...
cargo component build --target wasm32-wasip2 --release
if %errorlevel% neq 0 (
        echo Build failed.
        exit /b %errorlevel%
)

cd ../guest-qs

echo Building the project...
cargo component build --target wasm32-wasip2 --release
if %errorlevel% neq 0 (
        echo Build failed.
        exit /b %errorlevel%
)