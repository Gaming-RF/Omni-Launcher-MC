# OmniLauncherMC Windows Installer
# Downloads pre-built binary from GitHub Releases (fast, no compile needed)
# Falls back to source build if no release available
# Run PowerShell as Administrator:
#   Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/Gaming-RF/Omni-Launcher-MC/main/install.ps1' -OutFile "$env:TEMP\olmc-install.ps1"; powershell -ExecutionPolicy Bypass -File "$env:TEMP\olmc-install.ps1"

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$REPO = "Gaming-RF/Omni-Launcher-MC"
$API = "https://api.github.com/repos/$REPO"
$INSTALL_DIR = "$env:LOCALAPPDATA\OmniLauncherMC"
$BIN_DIR = "$env:LOCALAPPDATA\OmniLauncherMC\bin"

function Write-Info  { param([string]$msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Ok    { param([string]$msg) Write-Host "[OK]   $msg" -ForegroundColor Green }
function Write-Warn  { param([string]$msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Fail  { param([string]$msg) Write-Host "[FAIL] $msg" -ForegroundColor Red; exit 1 }
function Write-Step  { param([string]$msg) Write-Host "`n>> $msg" -ForegroundColor White }

function Refresh-Path {
    $machinePath = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
    $userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = "$machinePath;$userPath"
}

function Find-Command {
    param([string]$name)
    Refresh-Path
    return [bool](Get-Command $name -ErrorAction SilentlyContinue)
}

# ── Header ──────────────────────────────────────────────────────────
Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  OmniLauncherMC — Windows Installer      " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# ── Try pre-built binary first ──────────────────────────────────────
Write-Step "Checking for pre-built release..."

$release = $null
$msiAsset = $null
$exeAsset = $null

try {
    # Get latest release (include prereleases for now)
    $headers = @{}
    if ($env:GITHUB_TOKEN) { $headers["Authorization"] = "Bearer $env:GITHUB_TOKEN" }
    $releases = Invoke-RestMethod -Uri "$API/releases" -Headers $headers -UseBasicParsing

    foreach ($r in $releases) {
        if ($r.assets) {
            foreach ($a in $r.assets) {
                if ($a.name -match '\.msi$' -and $a.name -match 'x64') {
                    $msiAsset = $a
                    $release = $r
                    break
                }
                if ($a.name -match '\.exe$' -and $a.name -match 'x64') {
                    $exeAsset = $a
                    $release = $r
                }
            }
        }
        if ($msiAsset) { break }
    }
} catch {
    Write-Warn "Could not check GitHub releases: $_"
}

if ($msiAsset -or $exeAsset) {
    $tagName = $release.tag_name
    Write-Ok "Found release $tagName"

    if ($msiAsset) {
        # MSI installer — best option
        Write-Info "Downloading $($msiAsset.name) ($([math]::Round($msiAsset.size / 1MB, 1)) MB)..."
        $msiPath = "$env:TEMP\$($msiAsset.name)"
        Invoke-WebRequest -Uri $msiAsset.browser_download_url -OutFile $msiPath -UseBasicParsing
        Write-Info "Installing MSI..."
        Start-Process msiexec.exe -ArgumentList "/i","$msiPath","/qn" -Wait
        Remove-Item $msiPath -Force -ErrorAction SilentlyContinue
        Refresh-Path
        Write-Ok "OmniLauncherMC $tagName installed via MSI"
    } elseif ($exeAsset) {
        # Standalone exe
        Write-Info "Downloading $($exeAsset.name) ($([math]::Round($exeAsset.size / 1MB, 1)) MB)..."
        New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
        $exePath = "$INSTALL_DIR\omni-launcher-mc.exe"
        Invoke-WebRequest -Uri $exeAsset.browser_download_url -OutFile $exePath -UseBasicParsing
        Write-Ok "Downloaded standalone binary"
    }
} else {
    Write-Warn "No pre-built release found. Building from source..."
    Write-Host ""

    # ── Source build requires: Git, Node, pnpm, Rust, VS Build Tools ──

    # Git
    Write-Step "Checking Git..."
    if (Find-Command "git") {
        Write-Ok "Git $(git --version)"
    } else {
        Write-Info "Downloading Git portable..."
        $gitDir = "$INSTALL_DIR\tools\git"
        $gitZip = "$env:TEMP\mingit.zip"
        New-Item -ItemType Directory -Force -Path "$INSTALL_DIR\tools" | Out-Null
        Invoke-WebRequest -Uri "https://github.com/git-for-windows/git/releases/latest/download/MinGit-2.47.1.2-64-bit.zip" -OutFile $gitZip -UseBasicParsing
        Expand-Archive -Path $gitZip -DestinationPath $gitDir -Force
        Remove-Item $gitZip -Force
        $env:Path = "$gitDir\cmd;$env:Path"
        Write-Ok "Git portable installed"
    }

    # VS Build Tools (minimal — just C++ compiler, no IDE)
    Write-Step "Checking C++ compiler..."
    $hasCompiler = $false
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $p = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
        if ($p) { $hasCompiler = $true }
    }
    if (Find-Command "cl.exe") { $hasCompiler = $true }

    if (-not $hasCompiler) {
        Write-Info "Installing minimal C++ build tools (~1.5 GB)..."
        Write-Warn "This is the minimum needed to compile Rust on Windows."
        $vsUrl = "https://aka.ms/vs/17/release/vs_BuildTools.exe"
        $vsExe = "$env:TEMP\vs_BuildTools.exe"
        Invoke-WebRequest -Uri $vsUrl -OutFile $vsExe -UseBasicParsing
        # Minimal: just C++ compiler + Windows SDK, no extras
        Start-Process -FilePath $vsExe -ArgumentList "--add","Microsoft.VisualStudio.Workload.VCTools","--includeRecommended","--passive","--wait" -Wait
        Remove-Item $vsExe -Force -ErrorAction SilentlyContinue
        Refresh-Path
        Write-Ok "C++ build tools installed"
    } else {
        Write-Ok "C++ compiler present"
    }

    # Rust
    Write-Step "Checking Rust..."
    if (Find-Command "rustc") {
        Write-Ok "Rust $(rustc --version)"
    } else {
        Write-Info "Installing Rust..."
        $rsInit = "$env:TEMP\rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rsInit -UseBasicParsing
        Start-Process -FilePath $rsInit -ArgumentList "-y","--default-toolchain","stable" -Wait -NoNewWindow
        Refresh-Path
        if (Test-Path "$env:USERPROFILE\.cargo\bin") { $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path" }
        if (Find-Command "rustc") { Write-Ok "Rust $(rustc --version)" } else { Write-Fail "Rust install failed" }
    }

    # Node.js
    Write-Step "Checking Node.js..."
    if (Find-Command "node") {
        Write-Ok "Node.js $(node --version)"
    } else {
        Write-Info "Downloading Node.js..."
        $nodeMsi = "$env:TEMP\node.msi"
        Invoke-WebRequest -Uri "https://nodejs.org/dist/v22.16.0/node-v22.16.0-x64.msi" -OutFile $nodeMsi -UseBasicParsing
        Start-Process msiexec.exe -ArgumentList "/i","$nodeMsi","/qn" -Wait
        Remove-Item $nodeMsi -Force -ErrorAction SilentlyContinue
        Refresh-Path
        if (Find-Command "node") { Write-Ok "Node.js $(node --version)" } else { Write-Fail "Node.js install failed" }
    }

    # pnpm
    Write-Step "Checking pnpm..."
    if (Find-Command "pnpm") {
        Write-Ok "pnpm $(pnpm --version)"
    } else {
        npm install -g pnpm 2>$null
        Refresh-Path
        if (Find-Command "pnpm") { Write-Ok "pnpm $(pnpm --version)" } else { Write-Fail "pnpm install failed. Run: npm install -g pnpm" }
    }

    # Clone + build
    Write-Step "Building from source..."
    $srcRepo = "https://github.com/Gaming-RF/Omni-Launcher-MC.git"
    if (Test-Path "$INSTALL_DIR\.git") {
        Push-Location $INSTALL_DIR; git fetch --all --tags 2>$null; git reset --hard origin/main; Pop-Location
    } else {
        if (Test-Path $INSTALL_DIR) { Remove-Item -Recurse -Force $INSTALL_DIR }
        git clone --depth 1 $srcRepo $INSTALL_DIR
    }
    Write-Ok "Source ready ($(git -C $INSTALL_DIR rev-parse --short HEAD))"

    Push-Location $INSTALL_DIR
    try { pnpm install --frozen-lockfile 2>$null } catch { pnpm install }
    pnpm build
    Pop-Location
    Write-Ok "Frontend built"

    Push-Location "$INSTALL_DIR\src-tauri"
    cargo build --release
    Pop-Location
    Write-Ok "Backend built"
}

# ── Set up shortcuts ────────────────────────────────────────────────
Write-Step "Setting up shortcuts..."

# Find the binary
$binary = $null
$msiExe = "${env:ProgramFiles}\OmniLauncherMC\omni-launcher-mc.exe"
$localExe = "$INSTALL_DIR\omni-launcher-mc.exe"
$releaseExe = "$INSTALL_DIR\src-tauri\target\release\omni-launcher-mc.exe"

if (Test-Path $msiExe) { $binary = $msiExe }
elseif (Test-Path $localExe) { $binary = $localExe }
elseif (Test-Path $releaseExe) { $binary = $releaseExe }

if (-not $binary) {
    Write-Fail "Could not find omni-launcher-mc.exe"
}

New-Item -ItemType Directory -Force -Path $BIN_DIR | Out-Null
Set-Content -Path "$BIN_DIR\omni-launcher-mc.bat" -Value "@echo off`n`"$binary`" %*"

$userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$BIN_DIR*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$userPath;$BIN_DIR", "User")
    $env:Path = "$env:Path;$BIN_DIR"
    Write-Ok "Added to PATH"
}

try {
    $desktopPath = [System.Environment]::GetFolderPath("Desktop")
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut("$desktopPath\OmniLauncherMC.lnk")
    $shortcut.TargetPath = $binary
    $shortcut.WorkingDirectory = Split-Path $binary
    $shortcut.Description = "OmniLauncherMC - Minecraft Launcher"
    $shortcut.Save()
    Write-Ok "Desktop shortcut"
} catch { Write-Warn "Could not create shortcut: $_" }

try {
    $startMenu = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\OmniLauncherMC"
    New-Item -ItemType Directory -Force -Path $startMenu | Out-Null
    $m = $shell.CreateShortcut("$startMenu\OmniLauncherMC.lnk")
    $m.TargetPath = $binary
    $m.WorkingDirectory = Split-Path $binary
    $m.Save()
    Write-Ok "Start Menu entry"
} catch {}

# ── Update script ───────────────────────────────────────────────────
$updateScript = @'
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$dir = "$env:LOCALAPPDATA\OmniLauncherMC"
$API = "https://api.github.com/repos/Gaming-RF/Omni-Launcher-MC/releases"

# Try downloading newer release first
try {
    $releases = Invoke-RestMethod -Uri $API -UseBasicParsing
    $latest = $releases | Where-Object { -not $_.draft } | Select-Object -First 1
    if ($latest) {
        $msi = $latest.assets | Where-Object { $_.name -match '\.msi$' -and $_.name -match 'x64' } | Select-Object -First 1
        if ($msi) {
            Write-Host "[INFO] Downloading $($latest.tag_name) MSI..." -ForegroundColor Cyan
            $p = "$env:TEMP\$($msi.name)"
            Invoke-WebRequest -Uri $msi.browser_download_url -OutFile $p -UseBasicParsing
            Start-Process msiexec.exe -ArgumentList "/i","$p","/qn" -Wait
            Remove-Item $p -Force
            Write-Host "[OK] Updated to $($latest.tag_name)" -ForegroundColor Green
            exit 0
        }
    }
} catch {}

# Fallback: source build
if (-not (Test-Path "$dir\.git")) { Write-Host "Not installed." -ForegroundColor Red; exit 1 }
Set-Location $dir
git fetch --all --tags 2>$null
$local = git rev-parse HEAD; $remote = git rev-parse origin/main
if ($local -eq $remote) { Write-Host "[OK] Already latest ($(git rev-parse --short HEAD))" -ForegroundColor Green; exit 0 }
git reset --hard origin/main
Write-Host "[INFO] Rebuilding..." -ForegroundColor Cyan
try { pnpm install --frozen-lockfile 2>$null } catch { pnpm install }
pnpm build; Set-Location src-tauri; cargo build --release
Write-Host "[OK] Updated!" -ForegroundColor Green
'@
Set-Content -Path "$BIN_DIR\omni-launcher-mc-update.ps1" -Value $updateScript
Set-Content -Path "$BIN_DIR\omni-launcher-mc-update.bat" -Value "@echo off`npowershell -ExecutionPolicy Bypass -File `"$BIN_DIR\omni-launcher-mc-update.ps1`""
Write-Ok "Update command: omni-launcher-mc-update"

# ── Done ────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "        Installation Complete!             " -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Run:           omni-launcher-mc" -ForegroundColor White
Write-Host "                 (or desktop shortcut)" -ForegroundColor Gray
Write-Host "  Update later:  omni-launcher-mc-update" -ForegroundColor White
Write-Host "  Source:        $INSTALL_DIR" -ForegroundColor White
Write-Host ""
