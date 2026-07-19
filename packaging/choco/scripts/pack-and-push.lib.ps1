# Pure functions used by pack-and-push.ps1. Dot-source for testing.
Set-StrictMode -Version 3.0
$ErrorActionPreference = 'Stop'

function Get-ChocoVersion {
    [CmdletBinding()]
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Tag)
    if ($Tag -notmatch '^v(\d+\.\d+\.\d+(-\S+)?)$') {
        throw "Tag '$Tag' does not match ^v<major>.<minor>.<patch>[-...]"
    }
    return $Matches[1]
}

function ConvertFrom-Sha256Sidecar {
    [CmdletBinding()]
    param([Parameter(Mandatory)][AllowEmptyString()][string]$Content)
    $trimmed = $Content.Trim()
    if ($trimmed -notmatch '^([a-fA-F0-9]{64})\b') {
        throw "sidecar content is not a valid sha256 line: '$trimmed'"
    }
    return $Matches[1].ToLower()
}

function Invoke-TemplateSubstitution {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$Content,
        [Parameter(Mandatory)][hashtable]$Substitutions
    )
    foreach ($key in $Substitutions.Keys) {
        # Use [string]::Replace — literal (non-regex) substitution.
        $Content = $Content.Replace($key, [string]$Substitutions[$key])
    }
    return $Content
}

function Invoke-Git {
    # Thin indirection so tests can Mock it. Real invocation forwards to the git CLI.
    [CmdletBinding()]
    param([Parameter(ValueFromRemainingArguments)][string[]]$GitArgs)
    $out = & git @GitArgs
    if ($LASTEXITCODE -ne 0) {
        throw "git $($GitArgs -join ' ') failed with exit $LASTEXITCODE"
    }
    return $out -join [Environment]::NewLine
}

function Get-ReleaseNotesFromTag {
    [CmdletBinding()]
    param([Parameter(Mandatory)][string]$Tag)

    $notes = Invoke-Git 'tag' '-l' '--format=%(contents:subject)%0a%0a%(contents:body)' $Tag
    $notes = $notes.Trim()
    if ($notes -eq '') {
        return "See https://github.com/sassman/amoxide-rs/releases/tag/$Tag"
    }
    if ($notes.Contains(']]>')) {
        throw "tag body for $Tag contains ']]>' which is illegal inside CDATA"
    }
    return $notes
}

function Get-Sha256FromUrl {
    [CmdletBinding()]
    param([Parameter(Mandatory)][string]$Url)
    # GitHub serves release sidecars as application/octet-stream, which makes
    # Invoke-WebRequest return $resp.Content as byte[] under PowerShell 7 —
    # that breaks the [string] param on ConvertFrom-Sha256Sidecar.
    # Invoke-RestMethod decodes the body to string for us.
    $body = Invoke-RestMethod -Uri $Url
    return ConvertFrom-Sha256Sidecar -Content $body
}
