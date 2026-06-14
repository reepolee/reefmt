# Auto-bump patch version in Cargo.toml
$cargoPath = Join-Path $PSScriptRoot "Cargo.toml"
$cargoContent = Get-Content $cargoPath -Raw
$versionRegex = [regex]::Match($cargoContent, 'version = "(\d+)\.(\d+)\.(\d+)"')
if ($versionRegex.Success) {
    $major = $versionRegex.Groups[1].Value
    $minor = $versionRegex.Groups[2].Value
    $patch = [int]$versionRegex.Groups[3].Value + 1
    $newVersion = "$major.$minor.$patch"
    $cargoContent = $cargoContent -replace 'version = "\d+\.\d+\.\d+"', "version = `"$newVersion`""
    Set-Content $cargoPath -Value $cargoContent
    Write-Host "Bumped version to $newVersion"
} else {
    Write-Host "ERROR: Could not find version in Cargo.toml"
    exit 1
}

cargo build --release 
# binary at ./target/release/reefmt
# optionally:
Copy-Item ./target/release/reefmt.exe .
# Remove build artifacts (binary was copied above)
Remove-Item ./target -Recurse -Force

# reefmt routes/authors/form.ree
# reefmt **/*.ree
# reefmt views/users/show.ree routes/home.ree

# ./target/release/reefmt routes/authors/form.ree
