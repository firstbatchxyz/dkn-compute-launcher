# inspired from: https://github.com/chaqchase/lla/blob/main/install.sh
# and converted to Powershell by LLMs
#
# use with command:
#
# ```bash
# powershell -c "irm dria.co/install.ps1|iex"
# ```
#
# here `irm` tells it to do `Invoke-RestMethod` and the piped `iex` tells it to `Invoke-Expression`,
# allowing it to run the script.

################# LOGGERS #################

# the methods here are compatible with older Powershell versions

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

################## LOGIC ##################

function Get-ReleaseName {
  $OS = "windows"
  $script:ARCH = "amd64"

  # handle 32-bit arch error
  if (-not [System.Environment]::Is64BitOperatingSystem) {
    Write-Error "32-bit architecture is not supported, the launcher only works on AMD64 systems."
    exit 1
  }
  
  # handle arm64 arch error
  if ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture -eq [System.Runtime.InteropServices.Architecture]::Arm64) {
    Write-Error "ARM64 architecture is not supported, the launcher only works on AMD64 systems."
    exit 1
  }
  
  $script:RELEASE_NAME = "dkn-compute-launcher-${OS}-${ARCH}.exe"
}

function Get-LatestVersion {
  $LATEST_RELEASE_URL = "https://api.github.com/repos/firstbatchxyz/dkn-compute-launcher/releases/latest"
  try {
    $response = Invoke-RestMethod -Uri $LATEST_RELEASE_URL
    $script:VERSION = $response.tag_name
    
    $script:VERSION = "v0.1.0-test" # FIXME: !!!
  }
  catch {
    Write-Error "Failed to fetch latest version"
    exit 1
  }
}

function Download-Binary {
  Write-Step "Downloading Dria Compute Launcher $VERSION for windows-$ARCH..."
  
  $DOWNLOAD_URL = "https://github.com/firstbatchxyz/dkn-compute-launcher/releases/download/${VERSION}/${RELEASE_NAME}"
  Write-Step "Downloading from $DOWNLOAD_URL"
  
  $TMP_DIR = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
  New-Item -ItemType Directory -Path $TMP_DIR | Out-Null
  
  try {
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile "$TMP_DIR\dkn-compute-launcher.exe"
    Write-Success "Downloaded launcher to $TMP_DIR"
    $script:TMP_DIR = $TMP_DIR
  }
  catch {
    Write-Error "Failed to download launcher"
    Remove-Item -Path $TMP_DIR -Recurse -Force
    exit 1
  }
}

# Globally installs the Launcher binary
function Install-Binary {
  # Create Program Files directory if it doesn't exist
  $installPath = "$env:ProgramFiles\Dria"
  New-Item -ItemType Directory -Force -Path $installPath | Out-Null

  try {
    # Move the binary to Program Files
    Move-Item "$TMP_DIR\dkn-compute-launcher.exe" "$installPath\dkn-compute-launcher.exe" -Force

    # Add to PATH if not already present
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    if ($currentPath -notlike "*$installPath*") {
      [Environment]::SetEnvironmentVariable(
        "Path",
        "$currentPath;$installPath",
        "Machine"
      )
    }

    # Cleanup temp directory
    Remove-Item -Path $TMP_DIR -Recurse -Force
    Write-Success "Installed globally to $installPath"
  }
  catch {
    Write-Error "Failed to install globally. Try running as administrator."
    exit 1
  }
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
