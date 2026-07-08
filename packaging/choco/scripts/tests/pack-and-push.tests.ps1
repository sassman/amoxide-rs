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
