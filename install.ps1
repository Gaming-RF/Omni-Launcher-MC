# OmniLauncherMC Windows Installer
# Builds from source — always latest
# Run: irm https://raw.githubusercontent.com/Gaming-RF/Omni-Launcher-MC/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$REPO = "https://github.com/Gaming-RF/Omni-Launcher-MC.git"
$INSTALL_DIR = "$env:LOCALAPPDATA\OmniLauncherMC"
$BIN_DIR = "$env:LOCALAPPDATA\OmniLauncherMC\bin"

function Write-Info { param($msg) Write-Host "[INFO] $msg" -ForegroundColor Cyan }
function Write-Ok   { param($msg) Write-Host "[OK]   $msg" -ForegroundColor Green }
function Write-Warn { param($msg) Write-Host "[WARN] $msg" -ForegroundColor Yellow }
function Write-Fail { param($msg) Write-Host "[FAIL] $msg" -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  OmniLauncherMC — Windows Installer    " -ForegroundColor Cyan
Write-Host "  Builds from source, always latest      " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# ── Check admin for deps install ────────────────────────────────────
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

# ── Install Chocolatey (if needed) ──────────────────────────────────
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Info "Installing Chocolatey package manager..."
    Set-ExecutionPolicy Bypass -Scope Process -Force
    [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
    Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    Write-Ok "Chocolatey installed"
} else {
    Write-Ok "Chocolatey already installed"
}

# ── Install Git ─────────────────────────────────────────────────────
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Info "Installing Git..."
    choco install git -y --no-progress
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    Write-Ok "Git installed"
} else {
    Write-Ok "Git already installed ($(git --version))"
}

# ── Install Rust ────────────────────────────────────────────────────
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Info "Installing Rust..."
    Invoke-RestMethod -Uri https://win.rustup.rs/x86_64 -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y --default-toolchain stable
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    $env:CARGO_HOME = "$env:USERPROFILE\.cargo"
    $env:Path += ";$env:CARGO_HOME\bin"
    Write-Ok "Rust installed"
} else {
    Write-Ok "Rust already installed ($(rustc --version))"
}

# ── Install Node.js ─────────────────────────────────────────────────
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Info "Installing Node.js 22..."
    choco install nodejs-lts -y --no-progress
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    Write-Ok "Node.js installed"
} else {
    Write-Ok "Node.js already installed ($(node --version))"
}

# ── Install pnpm ────────────────────────────────────────────────────
if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    Write-Info "Installing pnpm..."
    npm install -g pnpm
    Write-Ok "pnpm installed"
} else {
    Write-Ok "pnpm already installed ($(pnpm --version))"
}

# ── Install Visual Studio Build Tools ───────────────────────────────
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasBuildTools = $false
if (Test-Path $vsWhere) {
    $installPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($installPath) { $hasBuildTools = $true }
}

if (-not $hasBuildTools) {
    Write-Info "Installing Visual Studio Build Tools (required for Rust on Windows)..."
    Write-Warn "This will take several minutes and may show a UAC prompt."
    choco install visualstudio2022buildtools -y --no-progress --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --wait"
    Write-Ok "VS Build Tools installed"
} else {
    Write-Ok "VS Build Tools already present"
}

# ── Clone or update repo ────────────────────────────────────────────
if (Test-Path "$INSTALL_DIR\.git") {
    Write-Info "Updating existing clone..."
    Set-Location $INSTALL_DIR
    git fetch --all --tags
    git reset --hard origin/main
} else {
    Write-Info "Cloning repository..."
    if (Test-Path $INSTALL_DIR) { Remove-Item -Recurse -Force $INSTALL_DIR }
    git clone --depth 1 $REPO $INSTALL_DIR
    Set-Location $INSTALL_DIR
}
$commit = git rev-parse --short HEAD
Write-Ok "Source ready ($commit)"

# ── Build ───────────────────────────────────────────────────────────
Set-Location $INSTALL_DIR

Write-Info "Installing frontend dependencies..."
try { pnpm install --frozen-lockfile } catch { pnpm install }
Write-Ok "Frontend deps installed"

Write-Info "Building frontend..."
pnpm build
Write-Ok "Frontend built"

Write-Info "Building Tauri backend (release)... this may take a few minutes"
Set-Location "$INSTALL_DIR\src-tauri"
cargo build --release
Write-Ok "Backend built"

$binary = "$INSTALL_DIR\src-tauri\target\release\omni-launcher-mc.exe"
if (-not (Test-Path $binary)) {
    Write-Fail "Binary not found at $binary"
}
$size = [math]::Round((Get-Item $binary).Length / 1MB, 1)
Write-Ok "Binary: $binary (${size} MB)"

# ── Create launcher script ──────────────────────────────────────────
New-Item -ItemType Directory -Force -Path $BIN_DIR | Out-Null

$launcher = @"
@echo off
"$binary" %*
"@
Set-Content -Path "$BIN_DIR\omni-launcher-mc.bat" -Value $launcher

# Add to user PATH
$userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$BIN_DIR*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$userPath;$BIN_DIR", "User")
    $env:Path += ";$BIN_DIR"
    Write-Ok "Added $BIN_DIR to user PATH"
}

# ── Create desktop shortcut ─────────────────────────────────────────
$desktopPath = [System.Environment]::GetFolderPath("Desktop")
$shortcutPath = "$desktopPath\OmniLauncherMC.lnk"
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $binary
$shortcut.WorkingDirectory = "$INSTALL_DIR\src-tauri\target\release"
$shortcut.Description = "OmniLauncherMC - Minecraft Launcher"
$shortcut.Save()
Write-Ok "Desktop shortcut created"

# ── Create start menu entry ─────────────────────────────────────────
$startMenu = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\OmniLauncherMC"
New-Item -ItemType Directory -Force -Path $startMenu | Out-Null
$menuShortcut = "$startMenu\OmniLauncherMC.lnk"
$shortcut2 = $shell.CreateShortcut($menuShortcut)
$shortcut2.TargetPath = $binary
$shortcut2.WorkingDirectory = "$INSTALL_DIR\src-tauri\target\release"
$shortcut2.Description = "OmniLauncherMC - Minecraft Launcher"
$shortcut2.Save()
Write-Ok "Start menu entry created"

# ── Create update script ────────────────────────────────────────────
$updateScript = @'
Set-Location "$env:LOCALAPPDATA\OmniLauncherMC"
git fetch --all --tags
$local = git rev-parse HEAD
$remote = git rev-parse origin/main
if ($local -eq $remote) {
    Write-Host "[OK] Already latest ($(git rev-parse --short HEAD))" -ForegroundColor Green
    exit 0
}
git reset --hard origin/main
Write-Host "[INFO] Rebuilding..." -ForegroundColor Cyan
pnpm install --frozen-lockfile 2>$null; if ($LASTEXITCODE -ne 0) { pnpm install }
pnpm build
Set-Location src-tauri; cargo build --release
Write-Host "[OK] Updated!" -ForegroundColor Green
'@
Set-Content -Path "$BIN_DIR\omni-launcher-mc-update.ps1" -Value $updateScript
$updateBat = @"
@echo off
powershell -ExecutionPolicy Bypass -File "$BIN_DIR\omni-launcher-mc-update.ps1"
"@
Set-Content -Path "$BIN_DIR\omni-launcher-mc-update.bat" -Value $updateBat
Write-Ok "Update command: omni-launcher-mc-update"

# ── Done ────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "       Installation Complete!            " -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Run:           omni-launcher-mc (or use desktop shortcut)"
Write-Host "  Update later:  omni-launcher-mc-update"
Write-Host "  Source:        $INSTALL_DIR"
Write-Host ""
