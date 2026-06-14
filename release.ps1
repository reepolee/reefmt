# Release script for Windows.
# Builds the native binary and publishes it as a GitHub Release.
# Version is auto-bumped (patch) only when the tag for the current version doesn't exist yet.
#
# Usage: .\release.ps1 [-Draft]
#   -Draft  Create the release as a draft (default: published)
#
# Prerequisites:
#   - gh CLI (https://cli.github.com) — authenticated via `gh auth login`
#   - git
#
# Workflow (run on each machine after pushing code):
#   1. macOS (first): bash release.sh     -> bumps version, creates tag + release, uploads
#   2. Linux:          bash release.sh     -> builds, uploads to existing release
#   3. Windows:        .\release.ps1      -> builds, uploads to existing release

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
# Read current version from Cargo.toml
# ──────────────────────────────────────────────

$cargoContent = Get-Content "Cargo.toml" -Raw
$versionMatch = [regex]::Match($cargoContent, 'version = "(\d+\.\d+\.\d+)"')
if (-not $versionMatch.Success) {
    Write-Error "Could not find version in Cargo.toml"
    exit 1
}

$version = $versionMatch.Groups[1].Value
$binaryName = "$AppName-windows-x64.exe"

# ──────────────────────────────────────────────
# Decide: bump version or use existing
# ──────────────────────────────────────────────

$tag = "v$version"
$doBump = $false

$tagRemote = git ls-remote --tags origin $tag 2>$null
if ($tagRemote -match "refs/tags/$tag`$") {
    # Tag already exists -> this is a subsequent machine. Just build and upload.
    Write-Host "═══ reefmt release $version for Windows ═══"
    Write-Host "  (Tag $tag already released. Uploading binary only.)"
    $doBump = $false
} else {
    # Tag doesn't exist -> this is the first machine. Bump version, then build and release.
    $parts = $version -split '\.'
    $newVersion = "$($parts[0]).$($parts[1]).$([int]$parts[2] + 1)"

    Write-Host "═══ reefmt release $newVersion for Windows ═══"
    Write-Host "  (Bumping from $version -> $newVersion)"

    $cargoContent = $cargoContent -replace 'version = "\d+\.\d+\.\d+"', "version = `"$newVersion`""
    Set-Content "Cargo.toml" -Value $cargoContent

    $version = $newVersion
    $tag = "v$version"
    $doBump = $true
}

# ──────────────────────────────────────────────
# Build
# ──────────────────────────────────────────────

Write-Host "`n→ Building $binaryName..."
cargo build --release
Copy-Item ".\target\release\$binaryName" ".\$binaryName"

# ──────────────────────────────────────────────
# Commit version bump (first machine only)
# ──────────────────────────────────────────────

if ($doBump) {
    Write-Host "`n→ Committing version bump..."
    git add "Cargo.toml"
    git commit -m "Bump version to $version"
    Write-Host "  Committed: Bump version to $version"
}

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

# Push tag and (if bumped) the version bump commit together
Write-Host "  Pushing tag $tag to origin..."
git push origin $tag

if ($doBump) {
    Write-Host "  Pushing version bump commit..."
    git push origin HEAD
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
