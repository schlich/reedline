use modalkit::keybindings::{EdgeEvent, EdgeRepeat, InputBindings};

use super::commands::{MOVE_CHAR_LEFT, MOVE_CHAR_RIGHT, MOVE_LINE_DOWN, MOVE_LINE_UP};
use super::{HelixAction, HelixMachine, HelixMode, HelixStep, ESC};

#[derive(Default)]
pub(super) struct HelixBindings;

const BINDINGS: &[(HelixMode, char, HelixStep)] = &[
    (HelixMode::Insert, ESC, (None, Some(HelixMode::Normal))),
    (
        HelixMode::Normal,
        'h',
        (Some(HelixAction::Motion(MOVE_CHAR_LEFT)), None),
    ),
    (
        HelixMode::Normal,
        'l',
        (Some(HelixAction::Motion(MOVE_CHAR_RIGHT)), None),
    ),
    (
        HelixMode::Normal,
        'j',
        (Some(HelixAction::Motion(MOVE_LINE_DOWN)), None),
    ),
    (
        HelixMode::Normal,
        'k',
        (Some(HelixAction::Motion(MOVE_LINE_UP)), None),
    ),
    // v toggles between Normal and Select
    (HelixMode::Normal, 'v', (None, Some(HelixMode::Select))),
    (HelixMode::Select, 'v', (None, Some(HelixMode::Normal))),
    // Select mode has the same motion bindings as Normal
    (
        HelixMode::Select,
        'h',
        (Some(HelixAction::Motion(MOVE_CHAR_LEFT)), None),
    ),
    (
        HelixMode::Select,
        'l',
        (Some(HelixAction::Motion(MOVE_CHAR_RIGHT)), None),
    ),
    (
        HelixMode::Select,
        'j',
        (Some(HelixAction::Motion(MOVE_LINE_DOWN)), None),
    ),
    (
        HelixMode::Select,
        'k',
        (Some(HelixAction::Motion(MOVE_LINE_UP)), None),
    ),
];

impl InputBindings<char, HelixStep> for HelixBindings {
    fn setup(&self, machine: &mut HelixMachine) {
        for &(mode, key, ref step) in BINDINGS {
            machine.add_mapping(mode, &[(EdgeRepeat::Once, EdgeEvent::Key(key))], step);
        }
    }
}
