use amoxide::update::AppModel;
use std::collections::BTreeSet;

pub use crate::tree::ProjectTrustState;
pub use amoxide::AliasId;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    GlobalHeader,
    ProfileHeader,
    AliasItem,
    ProjectHeader,
    SubcommandProgramHeader,
    SubcommandGroupNode,
    SubcommandItem,
}

impl NodeKind {
    pub fn is_navigable(&self) -> bool {
        true
    }
    pub fn is_selectable(&self) -> bool {
        matches!(
            self,
            NodeKind::AliasItem
                | NodeKind::SubcommandProgramHeader
                | NodeKind::SubcommandGroupNode
                | NodeKind::SubcommandItem
        )
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub kind: NodeKind,
    pub alias_id: Option<AliasId>,
    pub alias_command: Option<String>,
    pub is_active: bool,
    pub label: String,
    /// Prefix string for tree connectors (e.g. "│ ", "  ", "├─", "╰─")
    /// Used by the view to render tree structure lines.
    pub prefix: String,
    /// Prefix for content lines under this node (alias lines, connector lines).
    pub content_prefix: String,
    /// Trust state for project header nodes; `None` for all other node kinds.
    pub project_trust: Option<ProjectTrustState>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AliasField {
    Name,
    Command,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubcommandField {
    Short,
    Long,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AliasTarget {
    Global,
    Profile(String),
    Project,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransferMode {
    Move,
    Copy,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextInputState {
    NewProfile(String),
    NewAlias {
        name: String,
        command: String,
        active_field: AliasField,
        /// Byte offset of the cursor within the active field's string.
        cursor: usize,
        target: AliasTarget,
    },
    EditProfile {
        original_name: String,
        name: String,
        error: Option<String>,
    },
    EditAlias {
        alias_id: AliasId,
        name: String,
        command: String,
        active_field: AliasField,
        /// Byte offset of the cursor within the active field's string.
        cursor: usize,
        error: Option<String>,
    },
    SubcommandInput {
        program: String,
        pairs: Vec<(String, String)>,
        active_pair: usize,
        active_field: SubcommandField,
        /// Byte offset of the cursor within the currently active field's string.
        cursor: usize,
        target: AliasTarget,
        original_key: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Transfer(TransferMode),
    TextInput(TextInputState),
    Confirm(ConfirmAction),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoveDestination {
    Global,
    Project,
    Profile(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    DeleteProfile(String),
    OverwriteAliases {
        aliases: Vec<AliasId>,
        destination: MoveDestination,
        transfer_mode: TransferMode,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Column {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuiMessage {
    CursorUp,
    CursorDown,
    JumpTop,
    JumpBottom,
    ToggleSelect,
    EnterMoveMode,
    ExecuteTransfer,
    CancelTransfer,
    EnterCopyMode,
    SwitchColumn,
    StartCreateProfile,
    StartAddAlias,
    DeleteItem,
    UseProfile,
    UseProfileWithPriority(usize),
    TextInputChar(char),
    TextInputBackspace,
    TextInputConfirm,
    EditItem,
    TextInputCancel,
    TextInputSwitchField,
    TextInputSwitchFieldBack,
    TextInputCursorLeft,
    TextInputCursorRight,
    ConfirmYes,
    ConfirmNo,
    ToggleTrust,
    Quit,
    Resize(u16, u16),
}

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 15;

// Tree connector characters
pub const TREE_BRANCH: &str = "├─";
pub const TREE_LAST: &str = "╰─";
pub const TREE_TRUNK: &str = "│ ";
pub const TREE_SPACE: &str = "  ";

// Icons
pub const ICON_GLOBAL: &str = "🌐 ";
pub const ICON_PROJECT: &str = "📁 ";
pub const ICON_ACTIVE: &str = "●";
pub const ICON_INACTIVE: &str = "○";
pub const ICON_SUBCOMMAND: &str = "◆";

// Cursor and selection markers
pub const MARKER_CURSOR: &str = "▸ ";
pub const MARKER_SELECTED: &str = "■ ";
pub const MARKER_NONE: &str = "  ";

pub struct TuiModel {
    pub app_model: AppModel,
    pub tree: Vec<TreeNode>,
    pub cursor: usize,
    pub selected: BTreeSet<AliasId>,
    pub mode: Mode,
    pub dest_tree: Vec<TreeNode>,
    pub dest_cursor: usize,
    pub active_column: Column,
    pub scroll_offset: usize,
    pub status_line: Option<String>,
}

impl TuiModel {
    pub fn new() -> anyhow::Result<Self> {
        let app_model = AppModel::default();
        let mut model = Self {
            app_model,
            tree: Vec::new(),
            cursor: 0,
            selected: BTreeSet::new(),
            mode: Mode::Normal,
            dest_tree: Vec::new(),
            dest_cursor: 0,
            active_column: Column::Left,
            scroll_offset: 0,
            status_line: None,
        };
        model.rebuild_tree();
        Ok(model)
    }

    pub fn rebuild_tree(&mut self) {
        self.tree = crate::tree::build_tree(&self.app_model);
        self.dest_tree = crate::tree::build_dest_tree(&self.app_model);
        if !self.tree.is_empty() {
            if self.cursor >= self.tree.len() {
                self.cursor = self.tree.len() - 1;
            }
            self.cursor = self.next_navigable(self.cursor).unwrap_or(0);
        }
    }

    pub fn next_navigable(&self, from: usize) -> Option<usize> {
        let tree = if self.active_column == Column::Left {
            &self.tree
        } else {
            &self.dest_tree
        };
        (from..tree.len()).find(|&i| tree[i].kind.is_navigable())
    }

    pub fn adjust_scroll(&mut self, visible_height: usize) {
        let cursor_line = self.estimate_line_for_cursor();
        // An alias takes 3 rendered lines (name + command + separator).
        // Scroll in chunks of 3 and keep at least 1 line of padding at edges.
        let padding = 1;
        let chunk = 1;

        if cursor_line < self.scroll_offset + padding {
            // Cursor too close to top — scroll up by a chunk
            self.scroll_offset = cursor_line.saturating_sub(padding);
            // Align to chunk boundary
            self.scroll_offset = (self.scroll_offset / chunk) * chunk;
        } else if cursor_line + padding >= self.scroll_offset + visible_height {
            // Cursor too close to bottom — scroll down by a chunk
            let target = cursor_line + padding + 1;
            self.scroll_offset = target.saturating_sub(visible_height);
            // Align to chunk boundary (round up)
            self.scroll_offset = self.scroll_offset.div_ceil(chunk) * chunk;
        }
    }

    pub fn estimate_line_for_cursor(&self) -> usize {
        // Each node renders as 1 line (compact style), plus separator lines between sections
        let mut line = 0;
        for (i, node) in self.tree.iter().enumerate() {
            if i == self.cursor {
                break;
            }
            line += 1;
            // Account for blank separator line after a section boundary:
            // either after the last alias of a non-empty section, or after an empty ProfileHeader
            let is_empty_profile_header = node.kind == NodeKind::ProfileHeader;
            let is_last_alias = node.kind == NodeKind::AliasItem;
            let is_subcommand_node = matches!(
                node.kind,
                NodeKind::SubcommandItem
                    | NodeKind::SubcommandGroupNode
                    | NodeKind::SubcommandProgramHeader
            );
            if is_last_alias || is_empty_profile_header || is_subcommand_node {
                let next_is_header = self.tree.get(i + 1).is_some_and(|n| {
                    matches!(
                        n.kind,
                        NodeKind::GlobalHeader
                            | NodeKind::ProjectHeader
                            | NodeKind::ProfileHeader
                            | NodeKind::SubcommandProgramHeader
                    )
                });
                if next_is_header {
                    line += 1;
                }
            }
        }
        line
    }
}

#[cfg(test)]
mod new_types_exist {
    use super::*;

    #[test]
    fn subcommand_node_kinds_exist() {
        let _ = NodeKind::SubcommandProgramHeader;
        let _ = NodeKind::SubcommandGroupNode;
        let _ = NodeKind::SubcommandItem;
        let f: SubcommandField = SubcommandField::Short;
        assert_eq!(f, SubcommandField::Short);
    }

    #[test]
    fn subcommand_input_state_exists() {
        let _ = TextInputState::SubcommandInput {
            program: "jj".into(),
            pairs: vec![("ab".into(), "abandon".into())],
            active_pair: 0,
            active_field: SubcommandField::Long,
            cursor: 0,
            target: AliasTarget::Global,
            original_key: None,
        };
    }

    #[test]
    fn tui_message_variants_exist() {
        let _ = TuiMessage::TextInputSwitchFieldBack;
        let _ = TuiMessage::TextInputCursorLeft;
        let _ = TuiMessage::TextInputCursorRight;
    }
}
