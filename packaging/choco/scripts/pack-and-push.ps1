<#
.SYNOPSIS
    Pack and push amoxide + amoxide-tui Chocolatey packages.

.DESCRIPTION
    Reads the signed git tag body for release notes, fetches sha256 sidecars
    from the corresponding GitHub Release, substitutes placeholders in nuspec
    and install-script templates, then packs and pushes nupkgs to
    community.chocolatey.org.

.PARAMETER Tag
    Full tag name (e.g. v0.11.0). Defaults to $env:GITHUB_REF_NAME.

.PARAMETER DryRun
    Pack only; skip `choco push`.

.PARAMETER StagingDir
    Working directory for staged package files and produced nupkgs.
    Defaults to packaging/choco/.staging/.

.PARAMETER Packages
    Package IDs to build. Defaults to amoxide and amoxide-tui.

.EXAMPLE
    ./pack-and-push.ps1 -Tag v0.11.0 -DryRun

.EXAMPLE
    $env:CHOCO_API_KEY = '<key>'; ./pack-and-push.ps1 -Tag v0.11.0
#>
[CmdletBinding()]
param(
    [string]$Tag = $env:GITHUB_REF_NAME,
    [switch]$DryRun,
    [string]$StagingDir,
    [string[]]$Packages = @('amoxide', 'amoxide-tui')
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = 'Stop'

. "$PSScriptRoot/pack-and-push.lib.ps1"

$ChocoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
if (-not $StagingDir) {
    $StagingDir = Join-Path $ChocoRoot '.staging'
}

$version = Get-ChocoVersion -Tag $Tag
$releaseNotes = Get-ReleaseNotesFromTag -Tag $Tag

Write-Host "version:     $version"
Write-Host "tag body:    $($releaseNotes.Length) chars"
Write-Host "packages:    $($Packages -join ', ')"
Write-Host "staging dir: $StagingDir"
Write-Host "dry run:     $($DryRun.IsPresent)"

$triples = [ordered]@{
    'X64'   = 'x86_64-pc-windows-msvc'
    'ARM64' = 'aarch64-pc-windows-msvc'
}

# Fetch all sidecars up front — fail fast if any is missing.
# In dry-run mode a missing sidecar (e.g. ARM64 on a pre-ARM64 release tag) is
# tolerated: a stub hash is used so template substitution can still be verified.
$stubSha = '0000000000000000000000000000000000000000000000000000000000000000'
$hashes = @{}
foreach ($pkg in $Packages) {
    $hashes[$pkg] = @{}
    foreach ($arch in $triples.Keys) {
        $triple = $triples[$arch]
        $url = "https://github.com/sassman/amoxide-rs/releases/download/$Tag/$pkg-$triple.zip.sha256"
        Write-Host "  fetch $url"
        try {
            $hashes[$pkg][$arch] = Get-Sha256FromUrl -Url $url
        } catch {
            if ($DryRun) {
                Write-Host "    (dry-run) sidecar missing or invalid; using stub hash"
                $hashes[$pkg][$arch] = $stubSha
            } else {
                throw
            }
        }
    }
}

# Reset staging.
if (Test-Path $StagingDir) {
    Remove-Item $StagingDir -Recurse -Force
}
New-Item -Path $StagingDir -ItemType Directory | Out-Null

foreach ($pkg in $Packages) {
    $srcDir = Join-Path $ChocoRoot $pkg
    $dstDir = Join-Path $StagingDir $pkg
    Copy-Item $srcDir $dstDir -Recurse

    $nuspecTpl = Join-Path $dstDir "$pkg.nuspec.template"
    $nuspec = Get-Content $nuspecTpl -Raw
    $nuspec = Invoke-TemplateSubstitution -Content $nuspec -Substitutions @{
        '$version$'         = $version
        '__RELEASE_NOTES__' = $releaseNotes
    }
    $nuspecOut = Join-Path $dstDir "$pkg.nuspec"
    Set-Content -Path $nuspecOut -Value $nuspec -NoNewline

    $installTpl = Join-Path $dstDir 'tools/chocolateyInstall.ps1.template'
    $install = Get-Content $installTpl -Raw
    $urlX64 = "https://github.com/sassman/amoxide-rs/releases/download/$Tag/$pkg-x86_64-pc-windows-msvc.zip"
    $urlArm = "https://github.com/sassman/amoxide-rs/releases/download/$Tag/$pkg-aarch64-pc-windows-msvc.zip"
    $install = Invoke-TemplateSubstitution -Content $install -Substitutions @{
        '__URL_X64__'   = $urlX64
        '__SHA_X64__'   = $hashes[$pkg]['X64']
        '__URL_ARM64__' = $urlArm
        '__SHA_ARM64__' = $hashes[$pkg]['ARM64']
    }
    $installOut = Join-Path $dstDir 'tools/chocolateyInstall.ps1'
    Set-Content -Path $installOut -Value $install -NoNewline

    # Remove templates so they don't ship inside the nupkg.
    Remove-Item $nuspecTpl
    Remove-Item $installTpl

    Push-Location $dstDir
    try {
        & choco pack $nuspecOut --outputdirectory $StagingDir
        if ($LASTEXITCODE -ne 0) { throw "choco pack failed for $pkg" }
    } finally {
        Pop-Location
    }

    $nupkg = Join-Path $StagingDir "$pkg.$version.nupkg"
    if (-not (Test-Path $nupkg)) {
        throw "expected nupkg not produced: $nupkg"
    }

    if ($DryRun) {
        Write-Host "  dry-run: skipping push for $nupkg"
    } else {
        if (-not $env:CHOCO_API_KEY) {
            throw "CHOCO_API_KEY env var not set — cannot push"
        }
        & choco push $nupkg --source 'https://push.chocolatey.org/' --api-key $env:CHOCO_API_KEY
        if ($LASTEXITCODE -ne 0) { throw "choco push failed for $nupkg" }
    }
}

Write-Host "done."
