Write-Host "Starting reefmt release process..."
.\release.ps1
Write-Host "Running bash release script..."
wsl bash -c "cd /mnt/c/Users/ales/code/labs/reefmt && bash release.sh"
Write-Host "Release process complete"
