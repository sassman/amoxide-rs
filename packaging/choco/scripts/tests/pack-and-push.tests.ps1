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
