//! Helix command catalog for reedline's hx mode.
//!
//! Each constant maps a Helix command name to a native `modalkit` motion target,
//! so keybindings can stay traceable to Helix docs without translation layers.
//! Reference: <https://github.com/helix-editor/helix/blob/master/book/src/keymap.md>

use modalkit::prelude::{Count, EditTarget, MoveDir1D, MoveType};

/// `move_char_left`
pub(super) const MOVE_CHAR_LEFT: EditTarget =
    EditTarget::Motion(MoveType::Column(MoveDir1D::Previous, false), Count::Contextual);

/// `move_char_right`
pub(super) const MOVE_CHAR_RIGHT: EditTarget =
    EditTarget::Motion(MoveType::Column(MoveDir1D::Next, false), Count::Contextual);

/// `move_visual_line_down`
pub(super) const MOVE_LINE_DOWN: EditTarget =
    EditTarget::Motion(MoveType::Line(MoveDir1D::Next), Count::Contextual);

/// `move_visual_line_up`
pub(super) const MOVE_LINE_UP: EditTarget =
    EditTarget::Motion(MoveType::Line(MoveDir1D::Previous), Count::Contextual);
