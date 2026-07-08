Write-Host "Starting reefmt local build..."
try {
	.\release.ps1
	Write-Host "release.ps1 completed"
} catch {
	Write-Error "release.ps1 failed: $_"
	exit 1
}
Write-Host "Local build complete"
