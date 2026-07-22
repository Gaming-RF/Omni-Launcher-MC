# OmniLauncherMC Windows Installer
# Builds from source — always latest
# Run PowerShell as Administrator:
#   Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/Gaming-RF/Omni-Launcher-MC/main/install.ps1' -OutFile "$env:TEMP\olmc-install.ps1"; powershell -ExecutionPolicy Bypass -File "$env:TEMP\olmc-install.ps1"

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$REPO = "https://github.com/Gaming-RF/Omni-Launcher-MC.git"
$INSTALL_DIR = "$env:LOCALAPPDATA\OmniLauncherMC"
$BIN_DIR = "$env:LOCALAPPDATA\OmniLauncherMC\bin"
$TOOLS_DIR = "$env:LOCALAPPDATA\OmniLauncherMC\tools"

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

function Find-Command {
    param([string]$name)
    Refresh-Path
    return [bool](Get-Command $name -ErrorAction SilentlyContinue)
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
    Write-Warn "Not running as Administrator."
    Write-Warn "Some steps may fail. For best results, right-click PowerShell -> Run as Administrator."
}

# ── Step 1: Git ─────────────────────────────────────────────────────
Write-Step "Checking Git..."
if (Find-Command "git") {
    Write-Ok "Git $(git --version)"
} else {
    # Download Git portable
    Write-Info "Downloading Git..."
    $gitUrl = "https://github.com/git-for-windows/git/releases/latest/download/MinGit-2.47.1.2-64-bit.zip"
    $gitZip = "$TOOLS_DIR\mingit.zip"
    $gitDir = "$TOOLS_DIR\git"
    New-Item -ItemType Directory -Force -Path $TOOLS_DIR | Out-Null
    try {
        Invoke-WebRequest -Uri $gitUrl -OutFile $gitZip -UseBasicParsing
        Expand-Archive -Path $gitZip -DestinationPath $gitDir -Force
        Remove-Item $gitZip -Force
        $env:Path = "$gitDir\cmd;$env:Path"
        [System.Environment]::SetEnvironmentVariable("Path", "$env:Path", "User")
        Write-Ok "Git portable installed"
    } catch {
        Write-Fail "Git install failed. Install manually: https://git-scm.com/download/win"
    }
}

# ── Step 2: Visual Studio Build Tools ───────────────────────────────
Write-Step "Checking C++ compiler..."
$hasCompiler = $false
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $installPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($installPath) { $hasCompiler = $true }
}
if (Find-Command "cl.exe") { $hasCompiler = $true }

if (-not $hasCompiler) {
    Write-Info "Installing Visual Studio 2022 Build Tools..."
    Write-Warn "This is a large download (~2-6 GB). Please wait..."

    # Try chocolatey first
    Refresh-Path
    if (Find-Command "choco") {
        try {
            choco install visualstudio2022buildtools -y --no-progress --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive --wait"
            $hasCompiler = $true
        } catch { }
    }

    # Fallback: direct VS installer
    if (-not $hasCompiler) {
        Write-Info "Downloading VS Build Tools installer..."
        $vsUrl = "https://aka.ms/vs/17/release/vs_BuildTools.exe"
        $vsExe = "$TOOLS_DIR\vs_BuildTools.exe"
        New-Item -ItemType Directory -Force -Path $TOOLS_DIR | Out-Null
        Invoke-WebRequest -Uri $vsUrl -OutFile $vsExe -UseBasicParsing
        Write-Info "Launching installer (this will take a while)..."
        Start-Process -FilePath $vsExe -ArgumentList "--add","Microsoft.VisualStudio.Workload.VCTools","--includeRecommended","--passive","--wait" -Wait
        Remove-Item $vsExe -Force -ErrorAction SilentlyContinue
        Refresh-Path
        $hasCompiler = $true
    }
    Write-Ok "VS Build Tools installed"
} else {
    Write-Ok "C++ compiler present"
}

# ── Step 3: Rust ────────────────────────────────────────────────────
Write-Step "Checking Rust..."
if (Find-Command "rustc") {
    Write-Ok "Rust $(rustc --version)"
} else {
    Write-Info "Installing Rust..."
    $rustupInit = "$TOOLS_DIR\rustup-init.exe"
    New-Item -ItemType Directory -Force -Path $TOOLS_DIR | Out-Null
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit -UseBasicParsing
    Start-Process -FilePath $rustupInit -ArgumentList "-y","--default-toolchain","stable" -Wait -NoNewWindow
    Refresh-Path
    $cargoBin = "$env:USERPROFILE\.cargo\bin"
    if (Test-Path $cargoBin) { $env:Path = "$cargoBin;$env:Path" }
    if (Find-Command "rustc") {
        Write-Ok "Rust $(rustc --version)"
    } else {
        Write-Fail "Rust install failed. Install manually: https://rustup.rs/"
    }
}

# ── Step 4: Node.js ─────────────────────────────────────────────────
Write-Step "Checking Node.js..."
if (Find-Command "node") {
    Write-Ok "Node.js $(node --version)"
} else {
    Write-Info "Installing Node.js 22 LTS..."
    # Try chocolatey
    Refresh-Path
    if (Find-Command "choco") {
        try {
            choco install nodejs-lts -y --no-progress
            Refresh-Path
        } catch { }
    }

    # Fallback: direct MSI download
    if (-not (Find-Command "node")) {
        Write-Info "Downloading Node.js MSI..."
        $nodeVer = "22.16.0"
        $nodeMsi = "$TOOLS_DIR\node.msi"
        $nodeUrl = "https://nodejs.org/dist/v$nodeVer/node-v$nodeVer-x64.msi"
        New-Item -ItemType Directory -Force -Path $TOOLS_DIR | Out-Null
        Invoke-WebRequest -Uri $nodeUrl -OutFile $nodeMsi -UseBasicParsing
        Start-Process msiexec.exe -ArgumentList "/i","$nodeMsi","/qn" -Wait
        Remove-Item $nodeMsi -Force -ErrorAction SilentlyContinue
        Refresh-Path
    }

    if (Find-Command "node") {
        Write-Ok "Node.js $(node --version)"
    } else {
        Write-Fail "Node.js install failed. Install manually: https://nodejs.org/"
    }
}

# ── Step 5: pnpm ────────────────────────────────────────────────────
Write-Step "Checking pnpm..."
if (Find-Command "pnpm") {
    Write-Ok "pnpm $(pnpm --version)"
} else {
    Write-Info "Installing pnpm..."
    try {
        # npm global install works without admin
        npm install -g pnpm 2>$null
        Refresh-Path
    } catch { }
    if (-not (Find-Command "pnpm")) {
        try {
            # Fallback: standalone installer
            Invoke-WebRequest -Uri "https://get.pnpm.io/install.ps1" -OutFile "$TOOLS_DIR\pnpm-install.ps1" -UseBasicParsing
            powershell -ExecutionPolicy Bypass -File "$TOOLS_DIR\pnpm-install.ps1"
            Refresh-Path
        } catch { }
    }
    if (Find-Command "pnpm") {
        Write-Ok "pnpm $(pnpm --version)"
    } else {
        Write-Fail "pnpm install failed. Run manually: npm install -g pnpm"
    }
}

# ── Step 6: Clone or update repo ────────────────────────────────────
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

# ── Step 7: Build frontend ──────────────────────────────────────────
Write-Step "Building frontend..."
Push-Location $INSTALL_DIR
try { pnpm install --frozen-lockfile 2>$null } catch { pnpm install }
Write-Ok "Dependencies installed"
pnpm build
Write-Ok "Frontend built"
Pop-Location

# ── Step 8: Build Rust backend ──────────────────────────────────────
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

# ── Step 9: Create shortcuts ────────────────────────────────────────
Write-Step "Setting up shortcuts..."

# Batch launcher
New-Item -ItemType Directory -Force -Path $BIN_DIR | Out-Null
Set-Content -Path "$BIN_DIR\omni-launcher-mc.bat" -Value "@echo off`n`"$binary`" %*"

# Add BIN_DIR to user PATH
$userPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$BIN_DIR*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$userPath;$BIN_DIR", "User")
    $env:Path = "$env:Path;$BIN_DIR"
    Write-Ok "Added to PATH"
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

# ── Step 10: Update script ──────────────────────────────────────────
$updateScript = @'
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$dir = "$env:LOCALAPPDATA\OmniLauncherMC"
if (-not (Test-Path "$dir\.git")) { Write-Host "Not installed." -ForegroundColor Red; exit 1 }
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
Write-Host "                 (or use the desktop shortcut)" -ForegroundColor Gray
Write-Host "  Update later:  omni-launcher-mc-update" -ForegroundColor White
Write-Host "  Source:        $INSTALL_DIR" -ForegroundColor White
Write-Host ""
Write-Host "  NOTE: Restart your terminal if 'omni-launcher-mc' is not found." -ForegroundColor Yellow
Write-Host ""
