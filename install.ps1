# Inspired from: https://github.com/chaqchase/lla/blob/main/install.sh
# This script is for Windows
#
# Use with command:
#
# ```bash
# powershell -c "irm dria.co/install.ps1|iex"
# ```
#
# Here `irm` tells it to do `Invoke-RestMethod` and the piped `iex` tells it to `Invoke-Expression`,
# allowing it to run the script.

################# COLORS #################
$PSStyle.Foreground.Blue = "`e[34m"
$PSStyle.Foreground.Green = "`e[32m"
$PSStyle.Foreground.Red = "`e[31m"
$PSStyle.Reset = "`e[0m"

function Write-Step {
  param([string]$message)
  Write-Host "$($PSStyle.Foreground.Blue)==>$($PSStyle.Reset) $message"
}

function Write-Success {
  param([string]$message)
  Write-Host "$($PSStyle.Foreground.Green)==>$($PSStyle.Reset) $message"
}

function Write-Error {
  param([string]$message)
  Write-Host "$($PSStyle.Foreground.Red)==>$($PSStyle.Reset) $message"
}

function Get-ReleaseName {
  $OS = "windows"
  $ARCH = if ([System.Environment]::Is64BitOperatingSystem) { "amd64" } else { "386" }
  
  # For ARM64 Windows
  if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq [System.Runtime.InteropServices.Architecture]::Arm64) {
    $ARCH = "arm64"
  }
  
  $script:RELEASE_NAME = "dkn-compute-binary-${OS}-${ARCH}.exe"
}

function Get-LatestVersion {
  $LATEST_RELEASE_URL = "https://api.github.com/repos/firstbatchxyz/dkn-compute-launcher/releases/latest"
  try {
    $response = Invoke-RestMethod -Uri $LATEST_RELEASE_URL
    $script:VERSION = $response.tag_name
  }
  catch {
    Write-Error "Failed to fetch latest version"
    exit 1
  }
}

function Download-Binary {
  Write-Step "Downloading Dria Compute Launcher $VERSION for Windows-$ARCH..."
  
  $DOWNLOAD_URL = "https://github.com/firstbatchxyz/dkn-compute-launcher/releases/download/${VERSION}/${RELEASE_NAME}"
  Write-Step "Downloading from $DOWNLOAD_URL"
  
  $TMP_DIR = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
  New-Item -ItemType Directory -Path $TMP_DIR | Out-Null
  
  try {
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile "$TMP_DIR\dkn-compute-launcher.exe"
    Write-Success "Downloaded binary to $TMP_DIR"
    $script:TMP_DIR = $TMP_DIR
  }
  catch {
    Write-Error "Failed to download binary"
    Remove-Item -Path $TMP_DIR -Recurse -Force
    exit 1
  }
}

function Install-Binary {
  Move-Item "$TMP_DIR\dkn-compute-launcher.exe" ".\dkn-compute-launcher.exe"
  Remove-Item -Path $TMP_DIR -Recurse -Force
}

function Main {
  Write-Step "Installing Dria Compute Launcher to $(Get-Location)"
  
  if (-not (Get-Command "Invoke-WebRequest" -ErrorAction SilentlyContinue)) {
    Write-Error "PowerShell Web Cmdlets are not available"
    exit 1
  }
  
  Get-ReleaseName
  Get-LatestVersion
  Download-Binary
  Install-Binary
  
  Write-Success "DKN Compute Launcher $VERSION has been installed successfully!"
  Write-Success "Run '.\dkn-compute-launcher.exe help' to see settings"
  Write-Success "Run '.\dkn-compute-launcher.exe start' to start a node!"
}

Main
