BeforeAll {
    . "$PSScriptRoot/../pack-and-push.lib.ps1"
}

Describe 'lib skeleton' {
    It 'loads without error' {
        $true | Should -BeTrue
    }
}

Describe 'Get-ChocoVersion' {
    It 'strips leading v from a stable tag' {
        Get-ChocoVersion -Tag 'v0.11.0' | Should -Be '0.11.0'
    }
    It 'preserves prerelease suffixes' {
        Get-ChocoVersion -Tag 'v0.11.0-rc.1' | Should -Be '0.11.0-rc.1'
    }
    It 'throws on missing v prefix' {
        { Get-ChocoVersion -Tag '0.11.0' } | Should -Throw
    }
    It 'throws on empty tag' {
        { Get-ChocoVersion -Tag '' } | Should -Throw
    }
    It 'throws on non-semver' {
        { Get-ChocoVersion -Tag 'v1.2' } | Should -Throw
    }
}

Describe 'ConvertFrom-Sha256Sidecar' {
    It 'extracts hex from cargo-dist sidecar format' {
        $content = 'a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90  amoxide-x86_64-pc-windows-msvc.zip'
        ConvertFrom-Sha256Sidecar -Content $content | Should -Be 'a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90'
    }
    It 'normalises hex to lowercase' {
        $content = 'A1B2C3D4E5F60718293A4B5C6D7E8F90A1B2C3D4E5F60718293A4B5C6D7E8F90  file.zip'
        ConvertFrom-Sha256Sidecar -Content $content | Should -Be 'a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90'
    }
    It 'trims surrounding whitespace' {
        $content = "`n  a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90  file.zip`n"
        ConvertFrom-Sha256Sidecar -Content $content | Should -Be 'a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90'
    }
    It 'throws when hex is too short' {
        { ConvertFrom-Sha256Sidecar -Content 'deadbeef  file.zip' } | Should -Throw
    }
    It 'throws when content is empty' {
        { ConvertFrom-Sha256Sidecar -Content '' } | Should -Throw
    }
}

Describe 'Invoke-TemplateSubstitution' {
    It 'replaces a single sentinel' {
        $out = Invoke-TemplateSubstitution -Content 'hello __NAME__' -Substitutions @{ '__NAME__' = 'world' }
        $out | Should -Be 'hello world'
    }
    It 'replaces multiple sentinels in one pass' {
        $subs = @{
            '__A__' = '1'
            '__B__' = '2'
        }
        $out = Invoke-TemplateSubstitution -Content '__A__ and __B__' -Substitutions $subs
        $out | Should -Match '1 and 2'
    }
    It 'treats replacement values as literal strings (no regex)' {
        $out = Invoke-TemplateSubstitution -Content 'x = __VAL__' -Substitutions @{ '__VAL__' = '$1 + [42]' }
        $out | Should -Be 'x = $1 + [42]'
    }
    It 'replaces every occurrence of a repeated sentinel' {
        $out = Invoke-TemplateSubstitution -Content '__X__ __X__ __X__' -Substitutions @{ '__X__' = 'go' }
        $out | Should -Be 'go go go'
    }
    It 'passes content through when no sentinels match' {
        $out = Invoke-TemplateSubstitution -Content 'unchanged' -Substitutions @{ '__NONE__' = 'x' }
        $out | Should -Be 'unchanged'
    }
}

Describe 'Get-ReleaseNotesFromTag' {
    BeforeEach {
        # Mock `Invoke-Git` (thin wrapper we'll define alongside Get-ReleaseNotesFromTag)
        # so we don't need a real git tag during tests.
        Mock Invoke-Git { return 'v0.11.0 release' + [Environment]::NewLine + [Environment]::NewLine + 'body line one' + [Environment]::NewLine + 'body line two' } -ParameterFilter { $GitArgs[0] -eq 'tag' -and $GitArgs[-1] -eq 'v0.11.0' }
    }

    It 'returns trimmed tag body when non-empty' {
        $out = Get-ReleaseNotesFromTag -Tag 'v0.11.0'
        $out | Should -Match 'v0.11.0 release'
        $out | Should -Match 'body line two'
    }

    It 'falls back to release URL when tag body is empty' {
        Mock Invoke-Git { return '' } -ParameterFilter { $GitArgs[0] -eq 'tag' }
        Get-ReleaseNotesFromTag -Tag 'v0.11.0' | Should -Be 'See https://github.com/sassman/amoxide-rs/releases/tag/v0.11.0'
    }

    It 'falls back when tag body is whitespace only' {
        Mock Invoke-Git { return "   `n`n  " } -ParameterFilter { $GitArgs[0] -eq 'tag' }
        Get-ReleaseNotesFromTag -Tag 'v0.11.0' | Should -Be 'See https://github.com/sassman/amoxide-rs/releases/tag/v0.11.0'
    }

    It 'throws when tag body contains CDATA-end sequence' {
        Mock Invoke-Git { return 'harmless prefix ]]> harmless suffix' } -ParameterFilter { $GitArgs[0] -eq 'tag' }
        { Get-ReleaseNotesFromTag -Tag 'v0.11.0' } | Should -Throw '*]]>*'
    }
}
