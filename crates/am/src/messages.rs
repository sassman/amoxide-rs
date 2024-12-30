#[derive(Debug, PartialEq)]
pub enum Message<'a> {
    AddAlias(String, String),
    AddProfile(String, Option<String>),
    SetEnv(String),

    ListProfiles,
    LoadOrCreateProfile(&'a str, &'a Option<String>),
}
