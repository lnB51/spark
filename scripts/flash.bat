@echo off
set arg=%1

:: Check if an argument is provided; if not, default to "release"
if "%arg%"=="" (
    set arg=release
)

:: If the argument is "release", build the project in release mode
if "%arg%"=="release" (
    cargo build --release
) else if "%arg%"=="debug" (
    :: If the argument is "debug", build the project in debug mode
    cargo build
) else (
    :: If the argument is neither "release" nor "debug", show an error message and exit with an error code
    echo Wrong argument. Only "debug"/"release" arguments are supported.
    exit /b 1
)

:: Flash the compiled binary to the ESP32 and list all available serial ports
espflash flash target/riscv32imc-esp-espidf/debug/spark --list-all-ports