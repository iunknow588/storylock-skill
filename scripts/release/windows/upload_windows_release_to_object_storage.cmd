@echo off
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0upload_windows_release_to_object_storage.ps1" %*
