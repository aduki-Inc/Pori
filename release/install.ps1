# Pori Windows Installation Script
param(
    [string]$Version = "",
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Pori",
    [switch]$Force,
    [switch]$DryRun,
    [switch]$Help
)

$REPO = "aduki-Inc/Pori"
$BINARY_NAME = "pori.exe"

function Write-Info($message) {
    Write-Host "[INFO] $message" -ForegroundColor Blue
}

function Write-Success($message) {
    Write-Host "[SUCCESS] $message" -ForegroundColor Green
}

function Write-Warning($message) {
    Write-Host "[WARNING] $message" -ForegroundColor Yellow
}

function Write-Error($message) {
    Write-Host "[ERROR] $message" -ForegroundColor Red
}

function Show-Help {
    @"
Pori Windows Installation Script

USAGE:
    .\install.ps1 [OPTIONS]

OPTIONS:
    -Version VERSION     Install specific version (default: latest)
    -InstallDir DIR      Installation directory (default: %LOCALAPPDATA%\Programs\Pori)
    -Force               Force installation even if already exists
    -DryRun              Show what would be done without installing
    -Help                Show this help message

EXAMPLES:
    # Install latest version
    .\install.ps1

    # Install specific version
    .\install.ps1 -Version v0.1.4

    # Install to custom directory
    .\install.ps1 -InstallDir "C:\Tools\Pori"

    # Force reinstall
    .\install.ps1 -Force

REQUIREMENTS:
    - PowerShell 5.0 or later
    - Internet connection
    - Write permissions to installation directory

USAGE AFTER INSTALLATION:
    pori --url <WEBSOCKET_URL> --token <AUTH_TOKEN>

"@
}

function Get-LatestVersion {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest" -UseBasicParsing
        return $response.tag_name
    }
    catch {
        Write-Error "Failed to get latest version: $_"
        exit 1
    }
}

function Test-Command($command) {
    try {
        Get-Command $command -ErrorAction Stop | Out-Null
        return $true
    }
    catch {
        return $false
    }
}

function Install-Pori {
    if ($Help) {
        Show-Help
        return
    }

    Write-Info "Pori Windows Installation Script"
    Write-Info "Repository: https://github.com/$REPO"
    Write-Host ""

    if ($DryRun) {
        Write-Info "DRY RUN MODE - No changes will be made"
        Write-Host ""
    }

    # Check if already installed
    if ((Test-Command "pori") -and -not $Force) {
        try {
            $existingVersion = & pori --version 2>$null | Select-String "pori\s+(.+)" | ForEach-Object { $_.Matches[0].Groups[1].Value }
            if (-not $existingVersion) {
                $existingVersion = "unknown"
            }
        }
        catch {
            $existingVersion = "unknown"
        }
        Write-Warning "pori is already installed (version: $existingVersion)"
        Write-Info "Use -Force to reinstall"
        return
    }

    # Get version
    if (-not $Version) {
        Write-Info "Fetching latest version..."
        $Version = Get-LatestVersion
    }

    Write-Info "Installing pori $Version for Windows x86_64"

    if ($DryRun) {
        Write-Info "Would download from: https://github.com/$REPO/releases/download/$Version/pori-windows-x86_64.zip"
        Write-Info "Would install to: $InstallDir\$BINARY_NAME"
        Write-Info "Would add $InstallDir to user PATH"
        return
    }

    # Create installation directory
    if (-not (Test-Path $InstallDir)) {
        Write-Info "Creating installation directory: $InstallDir"
        try {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }
        catch {
            Write-Error "Failed to create installation directory: $_"
            exit 1
        }
    }

    # Setup temporary directory
    $tempDir = Join-Path $env:TEMP "pori-install-$(Get-Random)"
    $archivePath = Join-Path $tempDir "pori-windows-x86_64.zip"
    $downloadUrl = "https://github.com/$REPO/releases/download/$Version/pori-windows-x86_64.zip"

    if (Test-Path $tempDir) {
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    try {
        New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
    }
    catch {
        Write-Error "Failed to create temporary directory: $_"
        exit 1
    }

    # Download archive
    Write-Info "Downloading pori-windows-x86_64.zip..."
    try {
        # Use TLS 1.2 for GitHub compatibility
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $downloadUrl -OutFile $archivePath -UseBasicParsing
    }
    catch {
        Write-Error "Failed to download: $_"
        Write-Error "Please check your internet connection and try again"
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        exit 1
    }

    # Verify download
    if (-not (Test-Path $archivePath) -or (Get-Item $archivePath).Length -eq 0) {
        Write-Error "Downloaded file is empty or missing"
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        exit 1
    }

    # Extract archive
    Write-Info "Extracting archive..."
    try {
        Expand-Archive -Path $archivePath -DestinationPath $tempDir -Force
    }
    catch {
        Write-Error "Failed to extract archive: $_"
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        exit 1
    }

    # Find the binary
    $binaryPath = Join-Path $tempDir $BINARY_NAME
    if (-not (Test-Path $binaryPath)) {
        # Try to find it in subdirectories
        $foundBinary = Get-ChildItem -Path $tempDir -Name $BINARY_NAME -Recurse | Select-Object -First 1
        if ($foundBinary) {
            $binaryPath = Join-Path $tempDir $foundBinary
        }
        else {
            Write-Error "Binary '$BINARY_NAME' not found in archive"
            Write-Info "Archive contents:"
            Get-ChildItem -Path $tempDir -Recurse | ForEach-Object { Write-Info "  $($_.FullName)" }
            Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
            exit 1
        }
    }

    # Install binary
    $targetPath = Join-Path $InstallDir $BINARY_NAME
    Write-Info "Installing to $targetPath"
    try {
        Copy-Item $binaryPath $targetPath -Force
    }
    catch {
        Write-Error "Failed to copy binary: $_"
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        exit 1
    }

    # Add to user PATH
    try {
        $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($currentPath -notlike "*$InstallDir*") {
            Write-Info "Adding $InstallDir to user PATH"
            if ($currentPath) {
                $newPath = "$InstallDir;$currentPath"
            }
            else {
                $newPath = $InstallDir
            }
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            
            # Update current session PATH
            $env:Path = "$InstallDir;$env:Path"
            
            Write-Success "Added to PATH. Changes will take effect in new terminal sessions"
        }
        else {
            Write-Info "Directory already in PATH"
        }
    }
    catch {
        Write-Warning "Failed to add to PATH: $_"
        Write-Info "You may need to manually add $InstallDir to your PATH"
    }

    # Cleanup
    try {
        Remove-Item $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    catch {
        Write-Warning "Failed to cleanup temporary directory: $tempDir"
    }

    Write-Success "pori $Version installed successfully!"

    # Verify installation
    if (Test-Path $targetPath) {
        Write-Success "Verification successful!"
        
        # Test if binary works
        try {
            $versionOutput = & $targetPath --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Binary is working correctly: $versionOutput"
            }
            else {
                Write-Warning "Binary installed but may not be working correctly"
            }
        }
        catch {
            Write-Warning "Could not verify binary functionality: $_"
        }
        
        Write-Host ""
        Write-Info "USAGE:"
        Write-Info "    pori --url <WEBSOCKET_URL> --token <AUTH_TOKEN>"
        Write-Host ""
        Write-Info "EXAMPLES:"
        Write-Info "    pori --url wss://proxy.example.com --token your-token"
        Write-Info "    pori --config config.yml"
        Write-Host ""
        Write-Info "Run 'pori --help' for more options"
        Write-Host ""
        Write-Info "If 'pori' command is not found, restart your terminal or run:"
        Write-Info "    `$env:Path = [Environment]::GetEnvironmentVariable('Path', 'User')"
    }
    else {
        Write-Error "Installation verification failed - binary not found at $targetPath"
        exit 1
    }
}

# Main execution
try {
    Install-Pori
}
catch {
    Write-Error "Installation failed with unexpected error: $_"
    exit 1
}
