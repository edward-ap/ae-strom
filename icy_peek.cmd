@echo off
setlocal
chcp 65001 >nul
"%~dp0icy_peek.exe" %*
if errorlevel 1 pause
