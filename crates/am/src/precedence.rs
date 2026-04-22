use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::alias::{AliasName, AliasSet, TomlAlias};
use crate::subcommand::{SubcommandEntry, SubcommandSet};

#[derive(Debug, Clone, PartialEq)]
pub enum EntryKind {
    Alias(TomlAlias),
    SubcommandWrapper {
        program: String,
        entries: Vec<SubcommandEntry>,
        base_cmd: Option<String>,
    },
    /// Per-key subcommand entry tracked in `_AM_SUBCOMMANDS` for fine-grained
    /// change detection. Never emitted as shell code — the program-level
    /// `SubcommandWrapper` is the shell-visible unit.
    SubcommandKey {
        longs: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveEntry {
    pub name: String,
    pub kind: EntryKind,
    pub hash: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PrecedenceDiff {
    pub added: Vec<EffectiveEntry>,
    pub changed: Vec<EffectiveEntry>,
    pub removed: Vec<String>,
    pub unchanged: Vec<EffectiveEntry>,
}

#[derive(Debug, Default)]
pub struct Precedence {
    global_aliases: AliasSet,
    global_subcommands: SubcommandSet,
    profile_aliases: AliasSet,
    profile_subcommands: SubcommandSet,
    project_aliases: AliasSet,
    project_subcommands: SubcommandSet,
    shell_alias_state: BTreeMap<String, Option<String>>,
    shell_subcmd_state: BTreeMap<String, Option<String>>,
    external_functions: HashSet<String>,
    external_aliases: HashSet<String>,
}

impl Precedence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn resolve(self) -> PrecedenceDiff {
        PrecedenceDiff::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_produce_empty_diff() {
        let diff = Precedence::new().resolve();
        assert_eq!(diff, PrecedenceDiff::default());
    }
}
