@echo off
setlocal

REM git的安装目录
set GIT_PATH=C:\Program Files\Git\bin\git.exe

echo Updating Tables in: %1
cd %1
"%GIT_PATH%" pull

:end
endlocal