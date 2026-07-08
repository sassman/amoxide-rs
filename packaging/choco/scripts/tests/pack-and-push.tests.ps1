BeforeAll {
    . "$PSScriptRoot/../pack-and-push.lib.ps1"
}

Describe 'lib skeleton' {
    It 'loads without error' {
        $true | Should -BeTrue
    }
}
