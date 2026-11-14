@echo off
setlocal

rem Build the kwxRust quick test app. The drive script (tests\drive_quick.ps1)
rem launches target\debug\quick.exe and asserts its windows; this script only
rem builds it.
rem
rem   build_quick.cmd        build tests\quick (the verification gate)
rem   build_quick.cmd help   this text

cd /D "%~dp0"

if /I "%~1"=="help" goto :usage
if /I "%~1"=="-h" goto :usage
if /I "%~1"=="/?" goto :usage

if not exist "tests\quick\Cargo.toml" goto :no_manifest

cargo build --manifest-path tests\quick\Cargo.toml
if errorlevel 1 (
    echo ERROR: cargo build failed.
    exit /b 1
)

exit /b 0

:usage
echo Usage: build_quick.cmd
echo.
echo   build_quick.cmd        cargo build tests\quick and exit with its status
echo   build_quick.cmd help   this text
exit /b 0

:no_manifest
echo ERROR: tests\quick\Cargo.toml does not exist.
exit /b 1
