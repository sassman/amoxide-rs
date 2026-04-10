pub mod delegation;
mod navigation;
mod profile_actions;
mod selection;
mod text_input;
mod transfer;
mod trust;

use crate::model::{TuiMessage, TuiModel};

pub fn update(model: &mut TuiModel, msg: TuiMessage) {
    // Clear status line on each keypress
    model.status_line = None;

    match msg {
        TuiMessage::CursorUp
        | TuiMessage::CursorDown
        | TuiMessage::JumpTop
        | TuiMessage::JumpBottom
        | TuiMessage::SwitchColumn => navigation::handle(model, msg),

        TuiMessage::ToggleSelect => selection::handle_toggle(model),

        TuiMessage::StartCreateProfile
        | TuiMessage::StartAddAlias
        | TuiMessage::EditItem
        | TuiMessage::TextInputChar(_)
        | TuiMessage::TextInputBackspace
        | TuiMessage::TextInputConfirm
        | TuiMessage::TextInputCancel
        | TuiMessage::TextInputSwitchField => text_input::handle(model, msg),

        TuiMessage::EnterMoveMode
        | TuiMessage::EnterCopyMode
        | TuiMessage::ExecuteTransfer
        | TuiMessage::CancelTransfer => transfer::handle(model, msg),

        TuiMessage::DeleteItem
        | TuiMessage::ConfirmYes
        | TuiMessage::ConfirmNo
        | TuiMessage::UseProfile
        | TuiMessage::UseProfileWithPriority(_) => profile_actions::handle(model, msg),

        TuiMessage::ToggleTrust => trust::handle(model),

        TuiMessage::Quit | TuiMessage::Resize(_, _) => {} // handled at app layer
    }
}
