# Local build + install for Windows — NO releasing.
#
# The Mac mini cross-builds and publishes ALL release binaries (including the
# Windows ones) via release.sh. This script only builds reefmt natively on this
# Windows machine and installs it to ~/bin so your local `reefmt` stays current
# after a git pull. It never bumps the version, touches git, or uploads anything.
#
# Usage: .\release.ps1

$ErrorActionPreference = "Stop"
$AppName = "reefmt"

# Detect native arch for the target triple.
$arch = $env:PROCESSOR_ARCHITECTURE
$target = switch ($arch) {
    'AMD64' { "x86_64-pc-windows-msvc" }
    'ARM64' { "aarch64-pc-windows-msvc" }
    default { Write-Error "Unsupported architecture: $arch"; exit 1 }
}

Write-Host "═══ reefmt local build ($target) ═══"
Write-Host "  (Local install only — the Mac mini publishes releases.)"

Write-Host "`n→ Building..."
rustup target add $target 2>$null | Out-Null
cargo build --release --target $target
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed for $target"; exit 1
}

# ──────────────────────────────────────────────
# Install locally (to PATH)
# ──────────────────────────────────────────────

Write-Host "`n→ Installing locally..."
$InstallDir = Join-Path $HOME "bin"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item ".\target\$target\release\$AppName.exe" (Join-Path $InstallDir "$AppName.exe") -Force

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$Paths = $UserPath -split ";"
if ($Paths -notcontains $InstallDir) {
    $NewPath = if ([string]::IsNullOrWhiteSpace($UserPath)) {
        $InstallDir
    } else {
        "$UserPath;$InstallDir"
    }
    [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    Write-Host "  Added $InstallDir to user PATH"
    Write-Host "  Restart terminal to use $AppName"
}

Write-Host "  Installed to $(Join-Path $InstallDir "$AppName.exe")"

# Remove stale cargo-installed binary if present (avoids version conflicts)
$cargoBin = Join-Path $HOME ".cargo\bin\$AppName.exe"
if (Test-Path $cargoBin) {
    Remove-Item $cargoBin -Force
    Write-Host "  Removed stale $cargoBin"
}

Write-Host "`n✅ Done! Built + installed $AppName locally (no release)."
