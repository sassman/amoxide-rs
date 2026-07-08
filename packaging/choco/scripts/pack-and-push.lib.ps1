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
