use crate::shells::ShellBuilder;

#[derive(Clone)]
pub struct Alias(String);

impl Alias {
    pub fn from_last_command() -> anyhow::Result<Self> {
        ShellBuilder
            .build_current()?
            .last_command_from_history()
            .map(Self::try_from)?
    }
}

impl TryFrom<String> for Alias {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "$_" => Alias::from_last_command(),
            _ => Ok(Self(value)),
        }
    }
}

impl std::fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
