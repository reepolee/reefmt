# Release script for Windows.
# Builds the native binary and publishes it as a GitHub Release.
#
# Usage: .\release.ps1 [-Draft]
#   -Draft  Create the release as a draft (default: published)
#
# Prerequisites:
#   - gh CLI (https://cli.github.com) — authenticated via `gh auth login`
#   - git
#
# Workflow (run on each machine after pushing code):
#   1. macOS:  bash release.sh            -> builds, creates tag + release, uploads
#   2. Linux:  bash release.sh            -> builds, uploads to existing release
#   3. Windows: .\release.ps1            -> builds, uploads to existing release

param(
    [switch]$Draft
)

$ErrorActionPreference = "Stop"
$AppName = "reefmt"

# ──────────────────────────────────────────────
# Validate prerequisites
# ──────────────────────────────────────────────

if (-not (Get-Command "gh" -ErrorAction SilentlyContinue)) {
    Write-Error "gh CLI not found. Install it from https://cli.github.com/"
    exit 1
}

$authStatus = gh auth status 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error "gh CLI is not authenticated. Run: gh auth login"
    exit 1
}

# ──────────────────────────────────────────────
# Read version from Cargo.toml
# ──────────────────────────────────────────────

$cargoContent = Get-Content "Cargo.toml" -Raw
$versionMatch = [regex]::Match($cargoContent, 'version = "(\d+\.\d+\.\d+)"')
if (-not $versionMatch.Success) {
    Write-Error "Could not find version in Cargo.toml"
    exit 1
}

$version = $versionMatch.Groups[1].Value
$tag = "v$version"

Write-Host "═══ reefmt release $version for Windows ═══"

$binaryName = "$AppName-windows-x64.exe"

# ──────────────────────────────────────────────
# Build
# ──────────────────────────────────────────────

Write-Host "`n→ Building $binaryName..."
cargo build --release
Copy-Item ".\target\release\$binaryName" ".\$binaryName"

# ──────────────────────────────────────────────
# Create and push git tag
# ──────────────────────────────────────────────

Write-Host "`n→ Tagging $tag..."

$tagLocal = git rev-parse $tag 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "  Tag $tag already exists locally."
} else {
    git tag $tag
    Write-Host "  Created tag $tag locally."
}

# Push the tag if it hasn't been pushed yet
$tagRemote = git ls-remote --tags origin $tag 2>$null
if ($tagRemote -match $tag) {
    Write-Host "  Tag $tag already exists on origin."
} else {
    git push origin $tag
    Write-Host "  Pushed tag $tag to origin."
}

# ──────────────────────────────────────────────
# Create or upload to GitHub Release
# ──────────────────────────────────────────────

Write-Host "`n→ Publishing release $tag..."

$assetPath = ".\$binaryName"
$assetName = $binaryName
$releaseArgs = @()

$releaseExists = gh release view $tag 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "  Release $tag already exists. Uploading asset..."
    gh release upload $tag "$assetPath#$assetName" --clobber
} else {
    Write-Host "  Creating release $tag..."
    $releaseArgs = @(
        "create", $tag,
        "$assetPath#$assetName",
        "--title", $tag,
        "--notes", "Release $tag"
    )
    if ($Draft) {
        $releaseArgs += "--draft"
        Write-Host "  (Draft mode)"
    }
    gh @releaseArgs
}

# ──────────────────────────────────────────────
# Done
# ──────────────────────────────────────────────

$remoteUrl = git remote get-url origin
$repoPath = $remoteUrl -replace '.*github.com[/:]', '' -replace '\.git$', ''
Write-Host "`n✅ Done! Released $binaryName → $tag"
Write-Host "   View at: https://github.com/$repoPath/releases/tag/$tag"
