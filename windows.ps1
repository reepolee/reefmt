# Combined build + install script for Windows.
# Usage: .\windows.ps1

$AppName = "reefmt"

cargo build --release

# Copy binary from build output
Copy-Item ".\target\release\$AppName.exe" ".\$AppName.exe"

# Install
$BinName = "$AppName.exe"
$InstallDir = Join-Path $HOME "bin"
$Target = Join-Path $InstallDir $BinName

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item ".\$BinName" $Target -Force

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$Paths = $UserPath -split ";"

if ($Paths -notcontains $InstallDir) {
    $NewPath = if ([string]::IsNullOrWhiteSpace($UserPath)) {
        $InstallDir
    } else {
        "$UserPath;$InstallDir"
    }

    [Environment]::SetEnvironmentVariable(
        "Path",
        $NewPath,
        "User"
    )

    Write-Host "Added $InstallDir to user PATH"
} else {
    Write-Host "$InstallDir already in PATH"
}

Write-Host "Installed to $Target"
Write-Host ""
Write-Host "Restart terminal to use $AppName"

Write-Host ""
Write-Host "Usage:"
Write-Host "  reefmt routes/authors/form.ree"
Write-Host "  reefmt **/*.ree"
Write-Host "  reefmt views/users/show.ree routes/home.ree"

# Remove build artifacts
Remove-Item .\target -Recurse -Force
