use crate::model::{AliasId, Mode, NodeKind, TuiModel};

pub fn handle_toggle(model: &mut TuiModel) {
    if model.mode != Mode::Normal && !matches!(model.mode, Mode::Transfer(_)) {
        return;
    }
    if let Some(node) = model.tree.get(model.cursor).cloned() {
        if node.kind.is_selectable() {
            // Single alias toggle
            if let Some(ref id) = node.alias_id {
                if !model.selected.remove(id) {
                    model.selected.insert(id.clone());
                }
            }
        } else {
            // Header: toggle all aliases inside this container
            let child_ids = collect_child_aliases(model, model.cursor);
            if !child_ids.is_empty() {
                // If any are selected, deselect all. Otherwise select all.
                let any_selected = child_ids.iter().any(|id| model.selected.contains(id));
                if any_selected {
                    for id in &child_ids {
                        model.selected.remove(id);
                    }
                } else {
                    for id in child_ids {
                        model.selected.insert(id);
                    }
                }
            }
        }
    }
}

/// Collect all AliasItem ids that belong to the container at `header_idx`.
/// Walks forward from the header, collecting consecutive AliasItem nodes.
pub(super) fn collect_child_aliases(model: &TuiModel, header_idx: usize) -> Vec<AliasId> {
    let mut ids = Vec::new();
    for node in model.tree.iter().skip(header_idx + 1) {
        match node.kind {
            NodeKind::AliasItem => {
                if let Some(ref id) = node.alias_id {
                    ids.push(id.clone());
                }
            }
            // Stop at the next header — those aliases belong to a different container
            _ => break,
        }
    }
    ids
}
