# Run all SOC System components
# This script starts Rust, Python, and Flutter

Write-Host "=== Starting SOC System ===" -ForegroundColor Cyan

$projectRoot = Split-Path -Parent $PSScriptRoot
$rustRoot = "$projectRoot\rust-core"
$pythonRoot = "$projectRoot\python-ai"
$flutterRoot = "$projectRoot\flutter-app"

# Set environment variables
$env:OLLAMA_NUM_PARALLEL = 1
$env:OLLAMA_MAX_LOADED_MODELS = 1

# Check if Flutter is available
$flutterAvailable = Get-Command flutter -ErrorAction SilentlyContinue

# Start Ollama in background (if available)
if (Get-Command ollama -ErrorAction SilentlyContinue) {
    Write-Host "`nStarting Ollama..." -ForegroundColor Yellow
    Start-Process -FilePath "ollama" -ArgumentList "serve" -WindowStyle Hidden
    Start-Sleep -Seconds 2
} else {
    Write-Host "`nWarning: Ollama not found. Skipping..." -ForegroundColor Yellow
}

# Start Rust API Server
Write-Host "`nStarting Rust API Server (port 8080)..." -ForegroundColor Yellow
$rustAvailable = Get-Command cargo -ErrorAction SilentlyContinue
if ($rustAvailable) {
    Start-Process -FilePath "cargo" -ArgumentList "run,-p,api-server" -WorkingDirectory $rustRoot -PassThru -WindowStyle Normal
    Write-Host "Waiting for Rust API Server..." -ForegroundColor Yellow
    Start-Sleep -Seconds 5
} else {
    Write-Host "Warning: Rust/Cargo not found. Skipping..." -ForegroundColor Yellow
}

# Start Python FastAPI
Write-Host "`nStarting Python API (port 8000)..." -ForegroundColor Yellow
if (Test-Path "$pythonRoot\venv\Scripts\python.exe") {
    Set-Location $pythonRoot
    Start-Process -FilePath "$pythonRoot\venv\Scripts\python.exe" -ArgumentList "-m uvicorn main:app --host 0.0.0.0 --port 8000" -WorkingDirectory $pythonRoot -WindowStyle Normal
    Write-Host "Python API starting..." -ForegroundColor Yellow
    Start-Sleep -Seconds 3
} elseif (Get-Command python -ErrorAction SilentlyContinue) {
    Set-Location $pythonRoot
    python -m uvicorn main:app --host 0.0.0.0 --port 8000
} else {
    Write-Host "Warning: Python not found. Skipping..." -ForegroundColor Yellow
}

# Start Flutter Desktop if available
if ($flutterAvailable) {
    Write-Host "`nConfiguring Flutter Windows Desktop..." -ForegroundColor Yellow
    
    # Enable Windows desktop
    flutter config --enable-windows-desktop
    
    # Check if Windows project is properly configured (look for main.cpp)
    $windowsRunnerExists = Test-Path "$flutterRoot\windows\runner\main.cpp"
    
    if (!$windowsRunnerExists) {
        Write-Host "Creating Windows desktop project..." -ForegroundColor Yellow
        Set-Location $flutterRoot
        flutter create --platforms=windows .
    }
    
    # Run Flutter
    Write-Host "`nStarting Flutter Desktop..." -ForegroundColor Yellow
    Set-Location $flutterRoot
    flutter run -d windows
} else {
    Write-Host "`nWarning: Flutter not found." -ForegroundColor Red
    Write-Host "Please install Flutter from https://flutter.dev" -ForegroundColor Yellow
    Write-Host "Then run: flutter config --enable-windows-desktop" -ForegroundColor Yellow
    Write-Host "And run this script again." -ForegroundColor Yellow
}

Write-Host "`n=== SOC System Startup Complete ===" -ForegroundColor Cyan
