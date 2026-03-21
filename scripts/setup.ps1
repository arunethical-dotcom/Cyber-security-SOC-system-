# Setup script for SOC System
# This script installs all required dependencies

Write-Host "=== SOC System Setup ===" -ForegroundColor Cyan

# Check Rust
Write-Host "`nChecking Rust..." -ForegroundColor Yellow
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    $rustVersion = cargo --version
    Write-Host "Rust found: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "Rust not found. Please install from: https://rustup.rs" -ForegroundColor Red
    exit 1
}

# Check Python
Write-Host "`nChecking Python..." -ForegroundColor Yellow
if (Get-Command python -ErrorAction SilentlyContinue) {
    $pythonVersion = python --version
    Write-Host "Python found: $pythonVersion" -ForegroundColor Green
    
    # Check Python version is 3.11+
    $version = [int]((python --version).Split(" ")[1].Split(".")[0])
    if ($version -lt 3 -or $version -eq 3 -and [int]((python --version).Split(" ")[1].Split(".")[1]) -lt 11) {
        Write-Host "Python 3.11+ required. Found: $(python --version)" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Python not found. Please install Python 3.11+ from python.org" -ForegroundColor Red
    exit 1
}

# Check Flutter
Write-Host "`nChecking Flutter..." -ForegroundColor Yellow
if (Get-Command flutter -ErrorAction SilentlyContinue) {
    $flutterVersion = flutter --version | Select-Object -First 1
    Write-Host "Flutter found: $flutterVersion" -ForegroundColor Green
} else {
    Write-Host "Flutter not found. Please install from flutter.dev" -ForegroundColor Red
    exit 1
}

# Check/Install Ollama
Write-Host "`nChecking Ollama..." -ForegroundColor Yellow
if (Get-Command ollama -ErrorAction SilentlyContinue) {
    Write-Host "Ollama found" -ForegroundColor Green
} else {
    Write-Host "Installing Ollama..." -ForegroundColor Yellow
    # Download and install Ollama for Windows
    Invoke-WebRequest -Uri "https://ollama.ai/install.ps1" -UseBasicParsing | Invoke-Expression
    if (Get-Command ollama -ErrorAction SilentlyContinue) {
        Write-Host "Ollama installed successfully" -ForegroundColor Green
    } else {
        Write-Host "Failed to install Ollama. Please install manually from ollama.ai" -ForegroundColor Red
        exit 1
    }
}

# Pull Qwen model
Write-Host "`nPulling Qwen 1.8b model..." -ForegroundColor Yellow
$env:OLLAMA_NUM_PARALLEL = 1
$env:OLLAMA_MAX_LOADED_MODELS = 1
ollama pull qwen:1.8b

# Create Python virtual environment
Write-Host "`nSetting up Python environment..." -ForegroundColor Yellow
$projectRoot = Split-Path -Parent $PSScriptRoot
python -m venv "$projectRoot\python-ai\venv"
& "$projectRoot\python-ai\venv\Scripts\Activate.ps1"
pip install -r "$projectRoot\python-ai\requirements.txt"
deactivate

# Get Flutter dependencies
Write-Host "`nGetting Flutter dependencies..." -ForegroundColor Yellow
Set-Location "$projectRoot\flutter-app"
flutter pub get

# Build Rust
Write-Host "`nBuilding Rust components..." -ForegroundColor Yellow
Set-Location "$projectRoot\rust-core"
cargo build --release

Write-Host "`n=== Setup Complete ===" -ForegroundColor Cyan
Write-Host "Run scripts\run_all.ps1 to start the system" -ForegroundColor Green
