mod bindings;
mod commands;

use modalkit::{
    keybindings::{BindingMachine, EmptyKeyState, InputKey, ModalMachine, Mode, ModeKeys},
    prelude::{EditTarget, MoveDir1D},
};

const ESC: char = '\u{1B}';

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
/// Modal states for the experimental Helix edit mode key machine.
pub enum HelixMode {
    #[default]
    Insert,
    Normal,
    Select,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Actions produced by the experimental Helix edit mode key machine.
pub enum HelixAction {
    Type(char),
    Motion(EditTarget),
    /// Helix-style "traverse" motion: moves head via the EditTarget and
    /// resets anchor 1 cell past the old head in the given direction.
    /// This produces non-overlapping selections on consecutive motions.
    TraverseMotion(EditTarget, MoveDir1D),
    #[default]
    NoOp,
}

impl HelixAction {
    fn motion_target(&self) -> Option<&EditTarget> {
        match self {
            HelixAction::Motion(target) => Some(target),
            HelixAction::TraverseMotion(target, _) => Some(target),
            _ => None,
        }
    }
}

type HelixStep = (Option<HelixAction>, Option<HelixMode>);
/// Modal keybinding machine used by reedline's experimental Helix edit mode.
pub type HelixMachine = ModalMachine<char, HelixStep>;

impl Mode<HelixAction, EmptyKeyState> for HelixMode {}

impl<K: InputKey> ModeKeys<K, HelixAction, EmptyKeyState> for HelixMode {
    fn unmapped(&self, key: &K, _: &mut EmptyKeyState) -> (Vec<HelixAction>, Option<HelixMode>) {
        match self {
            HelixMode::Normal | HelixMode::Select => (vec![], None),
            HelixMode::Insert => match key.get_char() {
                Some(c) => (vec![HelixAction::Type(c)], None),
                None => (vec![], None),
            },
        }
    }
}

#[cfg(test)]
#[cfg(feature = "hx")]
mod test {

    use super::bindings::HelixBindings;
    use super::commands::{
        APPEND_MODE, INSERT_MODE, MOVE_CHAR_LEFT, MOVE_CHAR_RIGHT, MOVE_NEXT_WORD_START,
        MOVE_PREV_WORD_START, MOVE_VISUAL_LINE_DOWN, MOVE_VISUAL_LINE_UP, TRAVERSE_NEXT_WORD,
        TRAVERSE_PREV_WORD,
    };
    use super::*;
    use modalkit::{
        actions::{EditAction, EditorActions},
        editing::{
            application::EmptyInfo,
            buffer::EditBuffer,
            context::EditContextBuilder,
            cursor::{Cursor, CursorGroup, CursorState},
            store::Store,
        },
        prelude::{MoveDir1D, TargetShape, ViewportContext},
    };

    use rstest::*;

    struct TestBuf {
        ebuf: EditBuffer<EmptyInfo>,
        gid: modalkit::editing::buffer::CursorGroupId,
        vwctx: ViewportContext<Cursor>,
        store: Store<EmptyInfo>,
    }

    impl TestBuf {
        fn new(s: &str, start: Cursor) -> Self {
            let mut ebuf = EditBuffer::new("".to_string());
            let gid = ebuf.create_group();
            let vwctx = ViewportContext::default();
            let store = Store::default();

            ebuf.set_text(s);

            let leader = CursorState::Selection(start.clone(), start, TargetShape::CharWise);
            ebuf.set_group(gid, CursorGroup::new(leader, vec![]));

            Self {
                ebuf,
                gid,
                vwctx,
                store,
            }
        }

        fn apply_motion(&mut self, action: &HelixAction) {
            match action {
                HelixAction::Motion(target) => {
                    let ectx = EditContextBuilder::default()
                        .target_shape(Some(TargetShape::CharWise))
                        .build();
                    let ctx = &(self.gid, &self.vwctx, &ectx);
                    self.ebuf
                        .edit(&EditAction::Motion, target, ctx, &mut self.store)
                        .unwrap();
                }
                HelixAction::TraverseMotion(target, dir) => {
                    let old_head = self.leader();

                    let ectx = EditContextBuilder::default()
                        .target_shape(Some(TargetShape::CharWise))
                        .build();
                    let ctx = &(self.gid, &self.vwctx, &ectx);
                    self.ebuf
                        .edit(&EditAction::Motion, target, ctx, &mut self.store)
                        .unwrap();

                    let new_head = self.leader();

                    // Anchor is 1 cell past old head in the motion direction
                    let anchor = match dir {
                        MoveDir1D::Next => Cursor::new(old_head.y, old_head.x + 1),
                        MoveDir1D::Previous => {
                            Cursor::new(old_head.y, old_head.x.saturating_sub(1))
                        }
                    };

                    // CursorState::Selection(cursor/head, anchor, shape)
                    let leader = CursorState::Selection(new_head, anchor, TargetShape::CharWise);
                    self.ebuf
                        .set_group(self.gid, CursorGroup::new(leader, vec![]));
                }
                _ => panic!("action should map to a motion"),
            }
        }

        fn leader(&mut self) -> Cursor {
            self.ebuf.get_leader(self.gid)
        }

        fn anchor(&mut self) -> Cursor {
            let sel = self.ebuf.get_leader_selection(self.gid).unwrap();
            let head = self.leader();
            // get_leader_selection returns sorted (min, max, shape).
            // Derive anchor: whichever sorted position is NOT the head.
            if sel.0 == head {
                sel.1.clone()
            } else {
                sel.0.clone()
            }
        }

        fn selection(&mut self) -> Option<(Cursor, Cursor, TargetShape)> {
            self.ebuf.get_leader_selection(self.gid)
        }
    }

    #[test]
    fn test_insert_mode_is_default() {
        assert_eq!(HelixMachine::empty().mode(), HelixMode::Insert);
    }

    #[test]
    fn test_escape_to_normal_mode() {
        let mut machine = HelixMachine::from_bindings::<HelixBindings>();
        machine.input_key(ESC);
        assert_eq!(machine.mode(), HelixMode::Normal);
    }

    #[fixture]
    fn normal_machine() -> HelixMachine {
        let mut machine = HelixMachine::from_bindings::<HelixBindings>();
        machine.input_key(ESC);
        let _ = machine.pop();
        machine
    }

    #[rstest]
    #[case('h', HelixAction::Motion(MOVE_CHAR_LEFT), Cursor::new(0, 1))]
    #[case('l', HelixAction::Motion(MOVE_CHAR_RIGHT), Cursor::new(0, 3))]
    fn test_move_char(
        mut normal_machine: HelixMachine,
        #[case] key: char,
        #[case] expected_action: HelixAction,
        #[case] end: Cursor,
    ) {
        normal_machine.input_key(key);
        let (action, _) = normal_machine.pop().unwrap();
        assert_eq!(action, expected_action);

        let mut tb = TestBuf::new("hello\n", Cursor::new(0, 2));
        tb.apply_motion(&action);
        assert_eq!(tb.leader(), end);
    }

    #[rstest]
    #[case('j', HelixAction::Motion(MOVE_VISUAL_LINE_DOWN), Cursor::new(2, 2))]
    #[case('k', HelixAction::Motion(MOVE_VISUAL_LINE_UP), Cursor::new(0, 2))]
    fn test_move_line(
        mut normal_machine: HelixMachine,
        #[case] key: char,
        #[case] expected_action: HelixAction,
        #[case] end: Cursor,
    ) {
        normal_machine.input_key(key);
        let (action, _) = normal_machine.pop().unwrap();
        assert_eq!(action, expected_action);

        let mut tb = TestBuf::new("hello\nworld\nfoo\n", Cursor::new(1, 2));
        tb.apply_motion(&action);
        assert_eq!(tb.leader(), end);
    }

    #[rstest]
    fn test_move_next_word_start_lands_on_last_whitespace(mut normal_machine: HelixMachine) {
        normal_machine.input_key('w');
        let (action, _) = normal_machine.pop().unwrap();

        let mut tb = TestBuf::new("one  two\n", Cursor::new(0, 0));
        tb.apply_motion(&action);

        assert_eq!(
            tb.leader(),
            Cursor::new(0, 4),
            "head should land on last whitespace"
        );
    }

    #[rstest]
    fn test_move_prev_word_start_lands_on_word_first_letter(mut normal_machine: HelixMachine) {
        normal_machine.input_key('b');
        let (action, _) = normal_machine.pop().unwrap();
        assert_eq!(
            action,
            HelixAction::TraverseMotion(MOVE_PREV_WORD_START, MoveDir1D::Previous)
        );

        let mut tb = TestBuf::new("one  two\n", Cursor::new(0, 7));
        tb.apply_motion(&action);
        assert_eq!(
            tb.leader(),
            Cursor::new(0, 5),
            "head should land on word start"
        );
    }

    #[rstest]
    fn test_b_resets_anchor_to_last_whitespace(mut normal_machine: HelixMachine) {
        // In Helix normal mode, `b` from the first letter of a word should:
        //   - move head to first letter of the *previous* word
        //   - reset anchor to the last whitespace cell between the two words
        //
        // "one  two\n"
        //  01234567
        // Start at 5 ('t' of "two"), press b → head=0 ('o' of "one"), anchor=4 (last space)
        normal_machine.input_key('b');
        let (action, _) = normal_machine.pop().unwrap();

        let mut tb = TestBuf::new("one  two\n", Cursor::new(0, 5));
        tb.apply_motion(&action);

        assert_eq!(
            tb.anchor(),
            Cursor::new(0, 4),
            "anchor should reset to last whitespace between words"
        );
        assert_eq!(
            tb.leader(),
            Cursor::new(0, 0),
            "head should land on first letter of previous word"
        );
    }

    #[rstest]
    fn test_w_resets_anchor_to_first_letter(mut normal_machine: HelixMachine) {
        // In Helix normal mode, `w` from the last whitespace should:
        //   - move head to the last whitespace before the *next* word boundary
        //   - reset anchor to the first letter of the word that was traversed
        //
        // "one  two  end\n"
        //  0123456789...
        // Start at 4 (last space before "two"), press w → head=9 (last space before "end"),
        // anchor=5 ('t' of "two")
        normal_machine.input_key('w');
        let (action, _) = normal_machine.pop().unwrap();

        let mut tb = TestBuf::new("one  two  end\n", Cursor::new(0, 4));
        tb.apply_motion(&action);

        assert_eq!(
            tb.anchor(),
            Cursor::new(0, 5),
            "anchor should reset to first letter of the traversed word"
        );
        assert_eq!(
            tb.leader(),
            Cursor::new(0, 9),
            "head should land on last whitespace before next word"
        );
    }

    #[test]
    fn test_cursor_always_has_selection() {
        let mut tb = TestBuf::new("hello\n", Cursor::new(0, 2));
        assert_eq!(
            tb.selection(),
            Some((Cursor::new(0, 2), Cursor::new(0, 2), TargetShape::CharWise))
        );
    }

    #[rstest]
    fn test_selection_survives_motion(mut normal_machine: HelixMachine) {
        normal_machine.input_key('l');
        let (action, _) = normal_machine.pop().unwrap();

        let mut tb = TestBuf::new("hello\n", Cursor::new(0, 2));
        tb.apply_motion(&action);

        let sel = tb.selection().unwrap();
        assert_eq!(sel.0, Cursor::new(0, 2), "anchor should stay at start");
        assert_eq!(sel.1, Cursor::new(0, 3), "head should have moved");
        assert_eq!(sel.2, TargetShape::CharWise, "shape should remain CharWise");
    }

    #[rstest]
    fn test_v_toggles_select_mode(mut normal_machine: HelixMachine) {
        normal_machine.input_key('v');
        let _ = normal_machine.pop();
        assert_eq!(normal_machine.mode(), HelixMode::Select);

        normal_machine.input_key('v');
        let _ = normal_machine.pop();
        assert_eq!(normal_machine.mode(), HelixMode::Normal);
    }

    #[rstest]
    fn test_i_enters_insert_mode(mut normal_machine: HelixMachine) {
        normal_machine.input_key('i');
        let (action, _) = normal_machine.pop().unwrap();
        assert_eq!(action, INSERT_MODE.0.clone().unwrap_or_default());
        assert_eq!(normal_machine.mode(), HelixMode::Insert);
    }

    #[rstest]
    fn test_a_enters_insert_mode_after_moving_right(mut normal_machine: HelixMachine) {
        normal_machine.input_key('a');
        let (action, _) = normal_machine.pop().unwrap();
        assert_eq!(action, APPEND_MODE.0.clone().unwrap_or_default());
        assert_eq!(normal_machine.mode(), HelixMode::Insert);
    }

    #[rstest]
    #[case("hello\n", Cursor::new(0, 1), &['l', 'l'], &[Cursor::new(0, 2), Cursor::new(0, 3)])]
    #[case("hello\nworld\n", Cursor::new(0, 2), &['j', 'l'], &[Cursor::new(1, 2), Cursor::new(1, 3)])]
    fn test_select_mode_anchor_fixed(
        mut normal_machine: HelixMachine,
        #[case] text: &str,
        #[case] start: Cursor,
        #[case] keys: &[char],
        #[case] expected_heads: &[Cursor],
    ) {
        normal_machine.input_key('v');
        let _ = normal_machine.pop();

        let mut tb = TestBuf::new(text, start.clone());

        for (key, expected_head) in keys.iter().zip(expected_heads) {
            normal_machine.input_key(*key);
            let (action, _) = normal_machine.pop().unwrap();
            tb.apply_motion(&action);

            let sel = tb.selection().unwrap();
            assert_eq!(sel.0, start, "anchor should stay fixed");
            assert_eq!(sel.1, *expected_head, "head should have moved");
        }
    }
}
