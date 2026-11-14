@echo off
setlocal

rem Build the kwxRust probe lane. probe.wxui generates probe_mainframe.rs and
rem probe_dlg.rs into tests\quick_probe\gen\; probe_main.rs is hand-written.
rem Probe output never goes to tests\quick.
rem
rem   build_probe.cmd        build tests\quick_probe (the probe gate)
rem   build_probe.cmd help   this text

cd /D "%~dp0"

if /I "%~1"=="help" goto :usage
if /I "%~1"=="-h" goto :usage
if /I "%~1"=="/?" goto :usage

if not exist "tests\quick_probe\Cargo.toml" goto :no_manifest

cargo build --manifest-path tests\quick_probe\Cargo.toml
if errorlevel 1 (
    echo ERROR: cargo build failed for the probe lane.
    exit /b 1
)

exit /b 0

:usage
echo Usage: build_probe.cmd
echo.
echo   build_probe.cmd        cargo build tests\quick_probe and exit with its status
echo   build_probe.cmd help   this text
exit /b 0

:no_manifest
echo ERROR: tests\quick_probe\Cargo.toml does not exist.
exit /b 1
