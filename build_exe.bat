@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo === Budowanie EduPlus.exe ===

rem --- Szukanie prawdziwego Pythona (pomijamy atrape ze Sklepu Microsoft) ---
set "PY="
py -3 --version >nul 2>nul && set "PY=py -3"
if not defined PY (python --version >nul 2>nul && set "PY=python")
if not defined PY if exist "%USERPROFILE%\PycharmProjects\WelcomeScreen\.venv\Scripts\python.exe" set PY="%USERPROFILE%\PycharmProjects\WelcomeScreen\.venv\Scripts\python.exe"
if not defined PY (
    echo Nie znaleziono dzialajacego Pythona.
    echo Zainstaluj go z python.org i zaznacz "Add python.exe to PATH".
    pause
    exit /b 1
)
echo Uzywam: %PY%

echo [1/4] Instalacja bibliotek...
%PY% -m pip install flask flask_sqlalchemy pyinstaller || (pause & exit /b 1)
%PY% -m pip install pywebview || echo Uwaga: nie zainstalowano pywebview - program otworzy sie w przegladarce.

echo [2/4] Pobieranie Bootstrapa (do dzialania offline)...
if not exist static mkdir static
powershell -NoProfile -Command "Invoke-WebRequest 'https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/css/bootstrap.min.css' -OutFile 'static\bootstrap.min.css'; Invoke-WebRequest 'https://cdn.jsdelivr.net/npm/bootstrap@5.3.0/dist/js/bootstrap.bundle.min.js' -OutFile 'static\bootstrap.bundle.min.js'" || (echo Nie udalo sie pobrac Bootstrapa. Sprawdz internet. & pause & exit /b 1)

echo [3/4] Budowanie programu...
set "EXTRA="
%PY% -c "import webview" >nul 2>nul && set "EXTRA=--collect-all webview"
%PY% -m PyInstaller --noconfirm --onefile --windowed --name EduPlus --add-data "static;static" %EXTRA% app.py || (pause & exit /b 1)

echo [4/4] Gotowe!  Plik: dist\EduPlus.exe
pause
