# SOC Event Simulation Script
# This script runs the event generator and feeds events to the Rust collector

param(
    [Parameter(Position=0)]
    [ValidateSet("random", "brute_force", "lateral_movement", "exfiltration", "privilege_escalation", "mixed")]
    [string]$Mode = "mixed",
    
    [Parameter(Position=1)]
    [float]$Interval = 1.0,
    
    [Parameter(Position=2)]
    [int]$Count = 0,
    
    [switch]$Continuous
)

$projectRoot = Split-Path -Parent $PSScriptRoot
$pythonRoot = "$projectRoot\python-ai"
$rustRoot = "$projectRoot\rust-core"
$eventsRoot = "$projectRoot\events"

Write-Host "=== SOC Event Simulation ===" -ForegroundColor Cyan
Write-Host "Mode: $Mode" -ForegroundColor Yellow
Write-Host "Interval: $Interval seconds" -ForegroundColor Yellow

# Check if Python venv exists
$pythonExe = if (Test-Path "$pythonRoot\venv\Scripts\python.exe") {
    "$pythonRoot\venv\Scripts\python.exe"
} else {
    "python"
}

# Generator output file
$outputFile = "$eventsRoot\streams\event_stream.jsonl"

# Function to start event generator
function Start-EventGenerator {
    param([string]$Mode, [float]$Interval, [int]$Count)
    
    Write-Host "`nStarting event generator..." -ForegroundColor Yellow
    
    $generatorScript = "$eventsRoot\generators\generate_events.py"
    
    $args = @(
        $generatorScript,
        "--mode", $Mode,
        "--interval", $Interval,
        "--output", $outputFile
    )
    
    if ($Count -gt 0) {
        $args += "--count"
        $args += $Count
    }
    
    if ($Continuous) {
        # Run generator in background, output to file
        $process = Start-Process -FilePath $pythonExe -ArgumentList $args -PassThru -WindowStyle Normal
        return $process
    } else {
        # Run and pipe to event-input
        return $null
    }
}

# Function to start event input (Rust collector)
function Start-EventInput {
    param([string]$InputFile)
    
    Write-Host "`nStarting event input processor..." -ForegroundColor Yellow
    
    # Check if cargo is available
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        $process = Start-Process -FilePath "cargo" -ArgumentList "run,--bin,event-input,--","--input",$InputFile -WorkingDirectory $rustRoot -PassThru -WindowStyle Normal
        return $process
    } else {
        Write-Host "Warning: Cargo not found. Using Python stdin mode." -ForegroundColor Yellow
        return $null
    }
}

# Check if we should use pipe mode (Python -> Rust stdin)
$usePipeMode = $false

if ($Continuous) {
    # Continuous mode: generator writes to file, event-input watches file
    Write-Host "`nRunning in CONTINUOUS mode..." -ForegroundColor Green
    Write-Host "Generator output: $outputFile" -ForegroundColor Gray
    
    # Start the generator
    $genProcess = Start-EventGenerator -Mode $Mode -Interval $Interval -Count $Count
    
    # Small delay
    Start-Sleep -Seconds 2
    
    # Start event-input to process the file
    $inputProcess = Start-EventInput -InputFile $outputFile
    
    Write-Host "`nSimulation running. Press Ctrl+C to stop." -ForegroundColor Cyan
    
    # Wait for processes
    if ($genProcess) {
        $genProcess.WaitForExit()
    }
    if ($inputProcess) {
        $inputProcess.WaitForExit()
    }
} else {
    # One-shot mode: generate and immediately process
    Write-Host "`nRunning in ONE-SHOT mode..." -ForegroundColor Green
    
    # Generate events to file
    Write-Host "Generating events..." -ForegroundColor Yellow
    & $pythonExe "$eventsRoot\generators\generate_events.py" --mode $Mode --interval 0 --count 100 --output $outputFile
    
    # Process events
    Write-Host "Processing events..." -ForegroundColor Yellow
    
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        cargo run --bin event-input -- --url http://localhost:8080/events --input $outputFile
    } else {
        Write-Host "Warning: Cargo not found. Events generated to: $outputFile" -ForegroundColor Yellow
        Write-Host "To process events, run: cargo run -p event-input -- --input $outputFile" -ForegroundColor Cyan
    }
}

Write-Host "`n=== Simulation Complete ===" -ForegroundColor Cyan
