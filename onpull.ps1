Write-Host "Starting reefmt release process..."
Write-Error "Debug: About to run release.ps1"
try {
    .\release.ps1
    Write-Host "release.ps1 completed"
} catch {
    Write-Error "release.ps1 failed: $_"
    exit 1
}
Write-Host "Running bash release script..."
try {
    wsl bash -c "cd /mnt/c/Users/ales/code/labs/reefmt && bash release.sh"
    Write-Host "Bash script completed"
} catch {
    Write-Error "Bash script failed: $_"
    exit 1
}
Write-Host "Release process complete"
