# inspired from: https://github.com/chaqchase/lla/blob/main/install.sh
# this script is for Windows

# Define colors for output
$Colors = @{
  NC    = [Console]::ResetColor
  Red   = [ConsoleColor]::Red
  Green = [ConsoleColor]::Green
  Blue  = [ConsoleColor]::Blue
}

function Write-Step {
  param([string]$message)
  Write-Host "==> " -ForegroundColor Blue -NoNewline
  Write-Host $message
}

function Write-Success {
  param([string]$message)
  Write-Host "==> " -ForegroundColor Green -NoNewline
  Write-Host $message
}

function Write-Error {
  param([string]$message)
  Write-Host "==> " -ForegroundColor Red -NoNewline
  Write-Host $message
}

function Detect-Platform {
  $OS = "windows"
  $ARCH = if ([Environment]::Is64BitOperatingSystem) {
    if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq [System.Runtime.InteropServices.Architecture]::Arm64) {
      "arm64"
    } else {
      "amd64"
    }
  } else {
    Write-Error "Unsupported architecture"
    exit 1
  }

  $script:PLATFORM = "dkn-compute-launcher-${OS}-${ARCH}"
}

function Get-LatestVersion {
  $LatestReleaseUrl = "https://api.github.com/repos/firstbatchxyz/dkn-compute-launcher/releases/latest"
  try {
    $Response = Invoke-RestMethod -Uri $LatestReleaseUrl
    $script:VERSION = $Response.tag_name
  }
  catch {
    Write-Error "Failed to fetch latest version"
    exit 1
  }
}

function Download-Binary {
  Write-Step "Downloading Dria Compute Launcher ${VERSION} for windows-${ARCH}..."
  
  $DownloadUrl = "https://github.com/firstbatchxyz/dkn-compute-launcher/releases/download/${VERSION}/${PLATFORM}.exe"
  $script:TmpDir = Join-Path $env:TEMP ([System.Guid]::NewGuid())
  New-Item -ItemType Directory -Path $TmpDir | Out-Null
  
  try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile (Join-Path $TmpDir "dkn-compute-launcher.exe")
    Write-Success "Downloaded binary to ${TmpDir}"
  }
  catch {
    Write-Error "Failed to download binary"
    Remove-Item -Path $TmpDir -Recurse -Force
    exit 1
  }
}

function Install-Binary {
  Move-Item -Path (Join-Path $TmpDir "dkn-compute-launcher.exe") -Destination ".\dkn-compute-launcher.exe"
  Remove-Item -Path $TmpDir -Recurse -Force
}

function Main {
  Write-Step "Installing Dria Compute Launcher to $(Get-Location)"
  
  if (-not (Get-Command "Invoke-WebRequest" -ErrorAction SilentlyContinue)) {
    Write-Error "WebRequest module is required but not available"
    exit 1
  }

  Detect-Platform
  Get-LatestVersion
  Download-Binary
  Install-Binary

  Write-Success "dkn-compute-launcher ${VERSION} has been installed successfully!"
  Write-Success "Run '.\dkn-compute-launcher.exe help' to see settings"
  Write-Success "Run '.\dkn-compute-launcher.exe start' to start a node!"
}

# Run the main function
Main
