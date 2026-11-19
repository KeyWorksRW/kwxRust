@echo off
setlocal

rem Build the kwxRust QuickPort app. The drive script (tests\drive_quick.ps1 -Target port)
rem launches the binary through "cargo run --manifest-path tests\quick_port\Cargo.toml" and
rem asserts its windows; this script only builds it. Derived from build_quick.cmd: same
rem cargo invocation, only the manifest path differs.
rem
rem   build_quick_port.cmd        build tests\quick_port (the port lane)
rem   build_quick_port.cmd help   this text

cd /D "%~dp0"

if /I "%~1"=="help" goto :usage
if /I "%~1"=="-h" goto :usage
if /I "%~1"=="/?" goto :usage

if not exist "tests\quick_port\Cargo.toml" goto :no_manifest

cargo build --manifest-path tests\quick_port\Cargo.toml
if errorlevel 1 (
    echo ERROR: cargo build failed.
    exit /b 1
)

exit /b 0

:usage
echo Usage: build_quick_port.cmd
echo.
echo   build_quick_port.cmd        cargo build tests\quick_port and exit with its status
echo   build_quick_port.cmd help   this text
exit /b 0

:no_manifest
echo ERROR: tests\quick_port\Cargo.toml does not exist.
exit /b 1
