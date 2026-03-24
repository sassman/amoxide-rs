use std::collections::BTreeSet;
use std::path::PathBuf;
use am::update::AppModel;
use am::ProjectAliases;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum AliasId {
    Global { alias_name: String },
    Profile { profile_name: String, alias_name: String },
    Project { alias_name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    GlobalHeader,
    ProfileHeader,
    AliasItem,
    ProjectHeader,
}

impl NodeKind {
    pub fn is_navigable(&self) -> bool {
        true
    }
    pub fn is_selectable(&self) -> bool {
        matches!(self, NodeKind::AliasItem)
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub kind: NodeKind,
    pub depth: u16,
    pub alias_id: Option<AliasId>,
    pub alias_command: Option<String>,
    pub is_active: bool,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Moving,
    TextInput(String),
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
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Column {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TuiMessage {
    CursorUp, CursorDown, JumpTop, JumpBottom,
    ToggleSelect, EnterMoveMode, ExecuteMove, CancelMove,
    SwitchColumn,
    StartCreateProfile, DeleteItem, SetActive,
    TextInputChar(char), TextInputBackspace, TextInputConfirm, TextInputCancel,
    ConfirmYes, ConfirmNo,
    Quit, Resize(u16, u16),
}

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 15;

pub struct TuiModel {
    pub app_model: AppModel,
    pub project_aliases: Option<ProjectAliases>,
    pub project_path: Option<PathBuf>,
    pub tree: Vec<TreeNode>,
    pub cursor: usize,
    pub selected: BTreeSet<AliasId>,
    pub mode: Mode,
    pub dest_tree: Vec<TreeNode>,
    pub dest_cursor: usize,
    pub active_column: Column,
    pub scroll_offset: usize,
}

impl TuiModel {
    pub fn new() -> anyhow::Result<Self> {
        let app_model = AppModel::default();
        let cwd = std::env::current_dir()?;
        let project_path = ProjectAliases::find_path(&cwd)?;
        let project_aliases = match &project_path {
            Some(path) => Some(ProjectAliases::load(path)?),
            None => None,
        };
        let mut model = Self {
            app_model, project_aliases, project_path,
            tree: Vec::new(), cursor: 0, selected: BTreeSet::new(),
            mode: Mode::Normal, dest_tree: Vec::new(), dest_cursor: 0,
            active_column: Column::Left, scroll_offset: 0,
        };
        model.rebuild_tree();
        Ok(model)
    }

    pub fn rebuild_tree(&mut self) {
        self.tree = crate::tree::build_tree(&self.app_model, self.project_aliases.as_ref());
        self.dest_tree = crate::tree::build_dest_tree(&self.app_model, self.project_aliases.is_some());
        if !self.tree.is_empty() {
            if self.cursor >= self.tree.len() {
                self.cursor = self.tree.len() - 1;
            }
            self.cursor = self.next_navigable(self.cursor).unwrap_or(0);
        }
    }

    pub fn next_navigable(&self, from: usize) -> Option<usize> {
        let tree = if self.active_column == Column::Left { &self.tree } else { &self.dest_tree };
        (from..tree.len()).find(|&i| tree[i].kind.is_navigable())
    }

    pub fn prev_navigable(&self, from: usize) -> Option<usize> {
        let tree = if self.active_column == Column::Left { &self.tree } else { &self.dest_tree };
        (0..=from).rev().find(|&i| tree[i].kind.is_navigable())
    }

    pub fn adjust_scroll(&mut self, visible_height: usize) {
        let cursor_line = self.estimate_line_for_cursor();
        if cursor_line < self.scroll_offset {
            self.scroll_offset = cursor_line;
        } else if cursor_line >= self.scroll_offset + visible_height {
            self.scroll_offset = cursor_line - visible_height + 1;
        }
    }

    fn estimate_line_for_cursor(&self) -> usize {
        let mut line = 0;
        for (i, node) in self.tree.iter().enumerate() {
            if i == self.cursor {
                break;
            }
            line += match node.kind {
                NodeKind::AliasItem => 3,
                _ => 1,
            };
        }
        line
    }
}
