#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    SaveConfig,
    SaveProfiles,
    AddLocalAlias {
        name: String,
        cmd: String,
        raw: bool,
    },
    RemoveLocalAlias {
        name: String,
    },
    Print(String),
    SaveSecurity,
}
