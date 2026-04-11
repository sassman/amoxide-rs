use crate::model::{Mode, NodeKind, TuiModel};

pub fn handle(model: &mut TuiModel) {
    if model.mode != Mode::Normal {
        return;
    }
    let node = match model.tree.get(model.cursor) {
        Some(n) => n.clone(),
        None => return,
    };
    if node.kind != NodeKind::ProjectHeader {
        return;
    }
    let is_trusted = model
        .app_model
        .project_trust()
        .map(|t| t.is_trusted())
        .unwrap_or(false);
    let msg = if is_trusted {
        amoxide::Message::Untrust { forget: false }
    } else {
        amoxide::Message::Trust
    };
    let _ = super::delegation::dispatch(model, msg);
    // rebuild_tree is called by dispatch on success
}
