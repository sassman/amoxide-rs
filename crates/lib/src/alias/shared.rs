use crate::shells::ShellBuilder;

#[derive(Clone)]
pub struct Alias(String);

impl Alias {
    pub fn from_last_command() -> anyhow::Result<Self> {
        Ok(ShellBuilder
            .build_current()?
            .last_command_from_history()?
            .into())
    }
}

impl From<String> for Alias {
    fn from(value: String) -> Self {
        match value.as_str() {
            "$_" => todo!(r#"implement the special case 'get last command from history'"#),
            _ => Self(value),
        }
    }
}

impl std::fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
