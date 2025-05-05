@echo off
setlocal enabledelayedexpansion

echo Building the project...
@REM cargo component build --target wasm32-wasip2 --release
@REM @REM rustc ./src/main.rs --target wasm32-wasip2
@REM if %errorlevel% neq 0 (
@REM         echo Build failed.
@REM         exit /b %errorlevel%
@REM )

echo Running with Wasmtime...
wasmtime -S inherit-network=y .\target\wasm32-wasip1\release\host.wasm 127 0 0 1 8080 127 0 0 1 8090 127 0 0 1 8100
@REM wasmtime -S inherit-network=y ./main.wasm 127 0 0 1 8080 127 0 0 1 8090 127 0 0 1 8100
