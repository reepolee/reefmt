Write-Host "Starting reefmt release process..."
try {
	.\release.ps1
	Write-Host "release.ps1 completed"
}
catch {
	Write-Error "release.ps1 failed: $_"
	exit 1
}
Write-Host "Running bash release script..."
try {
	wsl bash release.sh
	Write-Host "Bash script completed"
}
catch {
	Write-Error "Bash script failed: $_"
	exit 1
}
Write-Host "Release process complete"
