# inspired from: https://github.com/chaqchase/lla/blob/main/install.sh
# and converted to Powershell by LLMs
#
# use with command:
#
# ```bash
# powershell -c "irm https://dria.co/launcher.ps1 | iex"
#
# # or the direct link
# powershell -c "irm https://raw.githubusercontent.com/firstbatchxyz/dkn-compute-launcher/refs/heads/master/install.ps1 | iex"
# ```
#
# `irm` does `Invoke-RestMethod` and the piped `iex` does `Invoke-Expression`, allowing it to run the downloaded script.

################# LOGGERS #################

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

################## HELPER FUNCTIONS ##################

function Write-Env {
  param([String]$Key, [String]$Value)
  
  $RegisterKey = Get-Item -Path 'HKCU:'
  $EnvRegisterKey = $RegisterKey.OpenSubKey('Environment', $true)
  
  if ($null -eq $Value) {
    $EnvRegisterKey.DeleteValue($Key)
  } else {
    $RegistryValueKind = if ($Value.Contains('%')) {
      [Microsoft.Win32.RegistryValueKind]::ExpandString
    } elseif ($EnvRegisterKey.GetValue($Key)) {
      $EnvRegisterKey.GetValueKind($Key)
    } else {
      [Microsoft.Win32.RegistryValueKind]::String
    }
    $EnvRegisterKey.SetValue($Key, $Value, $RegistryValueKind)
  }
  
  # notify system of environment change
  [System.Environment]::SetEnvironmentVariable($Key, $Value, 'User')
}

function Get-Env {
  param([String] $Key)
  $RegisterKey = Get-Item -Path 'HKCU:'
  $EnvRegisterKey = $RegisterKey.OpenSubKey('Environment')
  $EnvRegisterKey.GetValue($Key, $null, [Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames)
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
    Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
    exit 1
  }
}

function Install-Binary {
  # create .dria directory in user's home folder
  # we use this instead of $LOCALAPPDATA to keep cross-platform locations similar
  $installPath = "$HOME\.dria\bin"
  New-Item -ItemType Directory -Force -Path $installPath | Out-Null

  try {
    # check if binary is in use
    $runningProcesses = Get-Process | Where-Object { $_.Path -eq "$installPath\dkn-compute-launcher.exe" }
    if ($runningProcesses.Count -gt 0) {
      Write-Error "Install Failed - The launcher is currently running. Please close it and try again."
      exit 1
    }

    # move the binary to .dria/bin directory
    Move-Item "$TMP_DIR\dkn-compute-launcher.exe" "$installPath\dkn-compute-launcher.exe" -Force

    # add to user PATH if not already present
    $currentPath = (Get-Env -Key "Path") -split ';'
    if ($currentPath -notcontains $installPath) {
      $newPath = ($currentPath + $installPath) -join ';'
      Write-Env -Key 'Path' -Value $newPath
      $env:PATH += ";$installPath"
    }
    Write-Success "Added $installPath to your PATH"

    # cleanup temp directory
    Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
    Write-Success "Installed to $installPath"
  }
  catch {
    Write-Error "Failed to install: $_"
    Remove-Item -Path $TMP_DIR -Recurse -Force -ErrorAction SilentlyContinue
    exit 1
  }
}

function Main {
  Write-Step "Installing Dria Compute Launcher"
  
  if (-not (Get-Command "Invoke-WebRequest" -ErrorAction SilentlyContinue)) {
    Write-Error "PowerShell Web Cmdlets are not available"
    exit 1
  }
  
  Get-ReleaseName
  Get-LatestVersion
  Download-Binary
  Install-Binary
  
  Write-Success "dkn-compute-launcher $VERSION has been installed successfully!"
  Write-Success "Restart your terminal, and then:"
  Write-Success "  'dkn-compute-launcher.exe help' to see available commands,"
  Write-Success "  'dkn-compute-launcher.exe start' to start a node!"
}

Main
