# OmniLauncherMC Windows Installer
# Builds from source — always latest
# Run in PowerShell (as Administrator):
#   iex "& { $(irm https://raw.githubusercontent.com/Gaming-RF/Omni-Launcher-MC/main/install.ps1) }"
# Or download and run (recommended):
#   Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/Gaming-RF/Omni-Launcher-MC/main/install.ps1' -OutFile "$env:TEMP\olmc-install.ps1"; powershell -ExecutionPolicy Bypass -File "$env:TEMP\olmc-install.ps1"

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$REPO = "https://github.com/Gaming-RF/Omni-Launcher-MC.git"
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

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Install-IfMissing {
    param([string]$name, [string]$command, [scriptblock]$installAction)
    if (Get-Command $command -ErrorAction SilentlyContinue) {
        Write-Ok "$name already installed"
        return $true
    }
    Write-Info "Installing $name..."
    try {
        & $installAction
        Refresh-Path
        if (Get-Command $command -ErrorAction SilentlyContinue) {
            Write-Ok "$name installed"
            return $true
        } else {
            Write-Warn "$name installed but not found in PATH yet (may need terminal restart)"
            return $false
        }
    } catch {
        Write-Warn "Failed to install $name : $_"
        return $false
    }
}

# ── Header ──────────────────────────────────────────────────────────
Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  OmniLauncherMC — Windows Installer      " -ForegroundColor Cyan
Write-Host "  Builds from source, always latest        " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

$isAdmin = Test-Admin
if (-not $isAdmin) {
    Write-Warn "Not running as Administrator. Some installs may require elevation."
    Write-Warn "If this fails, right-click PowerShell -> Run as Administrator."
    Write-Host ""
}

# ── Step 1: Chocolatey ──────────────────────────────────────────────
Write-Step "Checking package manager..."
if (-not (Get-Command choco -ErrorAction SilentlyContinue)) {
    Write-Info "Installing Chocolatey..."
    try {
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
        Invoke-Expression ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
        Refresh-Path
        Write-Ok "Chocolatey installed"
    } catch {
        Write-Fail "Chocolatey install failed. Install manually: https://chocolatey.org/install"
    }
} else {
    Write-Ok "Chocolatey $(choco --version)"
}

# ── Step 2: Git ─────────────────────────────────────────────────────
Write-Step "Checking Git..."
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    choco install git -y --no-progress --params "'/NoShellIntegration'"
    Refresh-Path
    Write-Ok "Git installed"
} else {
    Write-Ok "Git $(git --version)"
}

# ── Step 3: Visual Studio Build Tools ───────────────────────────────
Write-Step "Checking Visual Studio Build Tools..."
$hasBuildTools = $false
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $installPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($installPath) { $hasBuildTools = $true }
}

if (-not $hasBuildTools) {
    # Check if C++ compiler is available at all
    if (Get-Command cl.exe -ErrorAction SilentlyContinue) {
        $hasBuildTools = $true
        Write-Ok "C++ compiler found in PATH"
    }
}

if (-not $hasBuildTools) {
    Write-Info "Installing Visual Studio 2022 Build Tools (required for Rust on Windows)..."
    Write-Warn "This is a large download (~2-6 GB) and takes 5-15 minutes."
    try {
        choco install visualstudio2022buildtools -y --no-progress --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --wait"
        Refresh-Path
        Write-Ok "VS Build Tools installed"
    } catch {
        Write-Warn "VS Build Tools install may have failed. If Rust build fails, install manually:"
        Write-Warn "  https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    }
} else {
    Write-Ok "VS Build Tools present"
}

# ── Step 4: Rust ────────────────────────────────────────────────────
Write-Step "Checking Rust..."
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Info "Installing Rust..."
    $rustupInit = "$env:TEMP\rustup-init.exe"
    Invoke-RestMethod -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit
    Start-Process -FilePath $rustupInit -ArgumentList "-y","--default-toolchain","stable" -Wait -NoNewWindow
    Refresh-Path
    # Also add cargo to current session
    $cargoBin = "$env:USERPROFILE\.cargo\bin"
    if (Test-Path $cargoBin) { $env:Path = "$env:Path;$cargoBin" }
    if (Get-Command rustc -ErrorAction SilentlyContinue) {
        Write-Ok "Rust $(rustc --version)"
    } else {
        Write-Fail "Rust install failed. Install manually: https://rustup.rs/"
    }
} else {
    Write-Ok "Rust $(rustc --version)"
}

# ── Step 5: Node.js ─────────────────────────────────────────────────
Write-Step "Checking Node.js..."
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    choco install nodejs-lts -y --no-progress
    Refresh-Path
    if (Get-Command node -ErrorAction SilentlyContinue) {
        Write-Ok "Node.js $(node --version)"
    } else {
        Write-Fail "Node.js install failed. Install manually: https://nodejs.org/"
    }
} else {
    Write-Ok "Node.js $(node --version)"
}

# ── Step 6: pnpm ────────────────────────────────────────────────────
Write-Step "Checking pnpm..."
if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    # Use npm global install (works without admin)
    try {
        npm install -g pnpm
    } catch {
        # Last resort: use npx pnpm
        Write-Warn "npm -g failed, trying corepack..."
        corepack enable
        corepack prepare pnpm@latest --activate
    }
    if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
        Write-Fail "pnpm install failed. Run: npm install -g pnpm"
    }
    Write-Ok "pnpm $(pnpm --version)"
} else {
    Write-Ok "pnpm $(pnpm --version)"
}

# ── Step 7: Clone or update repo ────────────────────────────────────
Write-Step "Getting source code..."
if (Test-Path "$INSTALL_DIR\.git") {
    Write-Info "Updating existing clone..."
    Push-Location $INSTALL_DIR
    git fetch --all --tags 2>$null
    git reset --hard origin/main
    Pop-Location
} else {
    Write-Info "Cloning repository..."
    if (Test-Path $INSTALL_DIR) { Remove-Item -Recurse -Force $INSTALL_DIR }
    git clone --depth 1 $REPO $INSTALL_DIR
}
$commit = (git -C $INSTALL_DIR rev-parse --short HEAD)
Write-Ok "Source ready ($commit)"

# ── Step 8: Build frontend ──────────────────────────────────────────
Write-Step "Building frontend..."
Push-Location $INSTALL_DIR
try { pnpm install --frozen-lockfile 2>$null } catch { pnpm install }
Write-Ok "Dependencies installed"
pnpm build
Write-Ok "Frontend built"
Pop-Location

# ── Step 9: Build Rust backend ──────────────────────────────────────
Write-Step "Building Tauri backend (release)... this takes 3-10 minutes"
Push-Location "$INSTALL_DIR\src-tauri"
cargo build --release
Pop-Location
Write-Ok "Backend built"

$binary = "$INSTALL_DIR\src-tauri\target\release\omni-launcher-mc.exe"
if (-not (Test-Path $binary)) {
    Write-Fail "Binary not found at $binary"
}
$sizeMB = [math]::Round((Get-Item $binary).Length / 1MB, 1)
Write-Ok "Binary: ${sizeMB} MB"

# ── Step 10: Create shortcuts ───────────────────────────────────────
Write-Step "Setting up shortcuts..."

# Batch launcher in BIN_DIR
New-Item -ItemType Directory -Force -Path $BIN_DIR | Out-Null
Set-Content -Path "$BIN_DIR\omni-launcher-mc.bat" -Value "@echo off`n`"$binary`" %*"

# Add BIN_DIR to user PATH
$userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$BIN_DIR*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$userPath;$BIN_DIR", "User")
    $env:Path = "$env:Path;$BIN_DIR"
    Write-Ok "Added to PATH (restart terminal to pick up)"
}

# Desktop shortcut
try {
    $desktopPath = [System.Environment]::GetFolderPath("Desktop")
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut("$desktopPath\OmniLauncherMC.lnk")
    $shortcut.TargetPath = $binary
    $shortcut.WorkingDirectory = "$INSTALL_DIR\src-tauri\target\release"
    $shortcut.Description = "OmniLauncherMC - Minecraft Launcher"
    $shortcut.Save()
    Write-Ok "Desktop shortcut"
} catch {
    Write-Warn "Could not create desktop shortcut: $_"
}

# Start Menu
try {
    $startMenu = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\OmniLauncherMC"
    New-Item -ItemType Directory -Force -Path $startMenu | Out-Null
    $menuLnk = $shell.CreateShortcut("$startMenu\OmniLauncherMC.lnk")
    $menuLnk.TargetPath = $binary
    $menuLnk.WorkingDirectory = "$INSTALL_DIR\src-tauri\target\release"
    $menuLnk.Save()
    Write-Ok "Start Menu entry"
} catch {
    Write-Warn "Could not create Start Menu entry: $_"
}

# ── Step 11: Update script ──────────────────────────────────────────
$updateScript = @'
param()
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$dir = "$env:LOCALAPPDATA\OmniLauncherMC"
if (-not (Test-Path "$dir\.git")) { Write-Host "Not installed. Run installer first." -ForegroundColor Red; exit 1 }
Set-Location $dir
git fetch --all --tags 2>$null
$local = git rev-parse HEAD
$remote = git rev-parse origin/main
if ($local -eq $remote) {
    Write-Host "[OK] Already latest ($(git rev-parse --short HEAD))" -ForegroundColor Green
    exit 0
}
git reset --hard origin/main
Write-Host "[INFO] Rebuilding from $(git rev-parse --short HEAD)..." -ForegroundColor Cyan
try { pnpm install --frozen-lockfile 2>$null } catch { pnpm install }
pnpm build
Set-Location src-tauri; cargo build --release
Write-Host "[OK] Updated! Run: omni-launcher-mc" -ForegroundColor Green
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
Write-Host "                 (or use the desktop shortcut)" -ForegroundColor Gray
Write-Host "  Update later:  omni-launcher-mc-update" -ForegroundColor White
Write-Host "  Source:        $INSTALL_DIR" -ForegroundColor White
Write-Host ""
Write-Host "  NOTE: If 'omni-launcher-mc' is not found, restart your terminal" -ForegroundColor Yellow
Write-Host "        to pick up the PATH change." -ForegroundColor Yellow
Write-Host ""
