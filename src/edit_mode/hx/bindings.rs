use modalkit::keybindings::{EdgeEvent, EdgeRepeat, InputBindings};

use super::commands::{
    APPEND_MODE, INSERT_MODE, MOVE_CHAR_LEFT, MOVE_CHAR_RIGHT, MOVE_NEXT_WORD_START,
    MOVE_VISUAL_LINE_DOWN, MOVE_VISUAL_LINE_UP,
};
use super::{HelixAction, HelixMachine, HelixMode, HelixStep, ESC};

#[derive(Default)]
pub(super) struct HelixBindings;

const BINDINGS: &[(HelixMode, char, HelixStep)] = &[
    (HelixMode::Insert, ESC, (None, Some(HelixMode::Normal))),
    // Insert mode entry
    (HelixMode::Normal, 'i', INSERT_MODE),
    (HelixMode::Normal, 'a', APPEND_MODE),
    // v toggles between Normal and Select
    (HelixMode::Normal, 'v', (None, Some(HelixMode::Select))),
    (HelixMode::Select, 'v', (None, Some(HelixMode::Normal))),
];

const NORMAL_AND_SELECT_MOTION_BINDINGS: &[(char, HelixStep)] = &[
    ('h', (Some(HelixAction::Motion(MOVE_CHAR_LEFT)), None)),
    ('l', (Some(HelixAction::Motion(MOVE_CHAR_RIGHT)), None)),
    (
        'j',
        (Some(HelixAction::Motion(MOVE_VISUAL_LINE_DOWN)), None),
    ),
    ('k', (Some(HelixAction::Motion(MOVE_VISUAL_LINE_UP)), None)),
    ('w', (Some(HelixAction::Motion(MOVE_NEXT_WORD_START)), None)),
];

const NORMAL_AND_SELECT_MODES: &[HelixMode] = &[HelixMode::Normal, HelixMode::Select];

impl InputBindings<char, HelixStep> for HelixBindings {
    fn setup(&self, machine: &mut HelixMachine) {
        for &(mode, key, ref step) in BINDINGS {
            machine.add_mapping(mode, &[(EdgeRepeat::Once, EdgeEvent::Key(key))], step);
        }

        for &mode in NORMAL_AND_SELECT_MODES {
            for &(key, ref step) in NORMAL_AND_SELECT_MOTION_BINDINGS {
                machine.add_mapping(mode, &[(EdgeRepeat::Once, EdgeEvent::Key(key))], step);
            }
        }
    }
}
