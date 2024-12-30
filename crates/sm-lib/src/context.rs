use std::process::Command;

use crate::shells::Shell;

#[derive(Debug)]
pub struct Context {
    shell: Box<dyn Shell>,
}

impl Context {
    pub fn new(shell: Box<dyn Shell>) -> Self {
        Self { shell }
    }

    pub fn shell(&self) -> &dyn Shell {
        &*self.shell
    }

    pub fn cmd(&self, cmd: &str) -> anyhow::Result<String> {
        let sh = format!("{}", self.shell);
        Ok(String::from_utf8(
            Command::new(sh).arg("-c").arg(cmd).output()?.stdout,
        )?)
    }
}
