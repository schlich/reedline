//! Helix command catalog for reedline's hx mode.
//!
//! Each constant maps a Helix command name to a native `modalkit` motion target,
//! so keybindings can stay traceable to Helix docs without translation layers.
//! Reference: <https://github.com/helix-editor/helix/blob/master/book/src/keymap.md>

use modalkit::prelude::{Count, EditTarget, MoveDir1D, MoveType};

use super::{HelixAction, HelixMode, HelixStep};

/// `move_char_left`
pub(super) const MOVE_CHAR_LEFT: EditTarget = EditTarget::Motion(
    MoveType::Column(MoveDir1D::Previous, false),
    Count::Contextual,
);

/// `move_char_right`
pub(super) const MOVE_CHAR_RIGHT: EditTarget =
    EditTarget::Motion(MoveType::Column(MoveDir1D::Next, false), Count::Contextual);

/// `move_visual_line_down`
pub(super) const MOVE_LINE_DOWN: EditTarget =
    EditTarget::Motion(MoveType::Line(MoveDir1D::Next), Count::Contextual);

/// `move_visual_line_up`
pub(super) const MOVE_LINE_UP: EditTarget =
    EditTarget::Motion(MoveType::Line(MoveDir1D::Previous), Count::Contextual);

/// `insert_mode` (`i`): enter Insert with cursor before the current selection.
pub(super) const INSERT_MODE: HelixStep = (None, Some(HelixMode::Insert));

/// `append_mode` (`a`): enter Insert with cursor after the current selection.
pub(super) const APPEND_MODE: HelixStep = (
    Some(HelixAction::Motion(MOVE_CHAR_RIGHT)),
    Some(HelixMode::Insert),
);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_insert_mode_enters_before_selection() {
        let (action, mode) = INSERT_MODE.clone();
        // In our collapsed-selection model, "before selection" means no pre-insert motion.
        assert_eq!(action, None);
        assert_eq!(mode, Some(HelixMode::Insert));
    }

    #[test]
    fn test_append_mode_enters_after_selection() {
        let (action, mode) = APPEND_MODE.clone();
        // In our collapsed-selection model, "after selection" is encoded as one-char right motion.
        assert_eq!(action, Some(HelixAction::Motion(MOVE_CHAR_RIGHT)));
        assert_eq!(mode, Some(HelixMode::Insert));
    }
}
