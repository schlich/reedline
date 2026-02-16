use modalkit::{
    keybindings::{
        BindingMachine, EdgeEvent, EdgeRepeat, EmptyKeyState, InputBindings, InputKey,
        ModalMachine, Mode, ModeKeys,
    },
    prelude::{Count, EditTarget, MoveDir1D, MoveType},
};

const ESC: char = '\u{1B}';

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
enum HelixMode {
    #[default]
    Insert,
    Normal,
    Select,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum HelixAction {
    Type(char),
    MoveChar(MoveDir1D),
    MoveLine(MoveDir1D),
    #[default]
    NoOp,
}

impl TryFrom<HelixAction> for MoveType {
    type Error = ();

    fn try_from(action: HelixAction) -> Result<Self, ()> {
        match action {
            HelixAction::MoveChar(dir) => Ok(MoveType::Column(dir, false)),
            HelixAction::MoveLine(dir) => Ok(MoveType::Line(dir)),
            _ => Err(()),
        }
    }
}

impl HelixAction {
    fn to_edit_target(&self, count: Count) -> Option<EditTarget> {
        MoveType::try_from(*self)
            .ok()
            .map(|mv| EditTarget::Motion(mv, count))
    }
}

type HelixStep = (Option<HelixAction>, Option<HelixMode>);
pub type HelixMachine = ModalMachine<char, HelixStep>;

#[derive(Default)]
struct HelixBindings;

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

const BINDINGS: &[(HelixMode, char, HelixStep)] = &[
    (HelixMode::Insert, ESC, (None, Some(HelixMode::Normal))),
    (
        HelixMode::Normal,
        'h',
        (Some(HelixAction::MoveChar(MoveDir1D::Previous)), None),
    ),
    (
        HelixMode::Normal,
        'l',
        (Some(HelixAction::MoveChar(MoveDir1D::Next)), None),
    ),
    (
        HelixMode::Normal,
        'j',
        (Some(HelixAction::MoveLine(MoveDir1D::Next)), None),
    ),
    (
        HelixMode::Normal,
        'k',
        (Some(HelixAction::MoveLine(MoveDir1D::Previous)), None),
    ),
    // v toggles between Normal and Select
    (HelixMode::Normal, 'v', (None, Some(HelixMode::Select))),
    (HelixMode::Select, 'v', (None, Some(HelixMode::Normal))),
    // Select mode has the same motion bindings as Normal
    (
        HelixMode::Select,
        'h',
        (Some(HelixAction::MoveChar(MoveDir1D::Previous)), None),
    ),
    (
        HelixMode::Select,
        'l',
        (Some(HelixAction::MoveChar(MoveDir1D::Next)), None),
    ),
    (
        HelixMode::Select,
        'j',
        (Some(HelixAction::MoveLine(MoveDir1D::Next)), None),
    ),
    (
        HelixMode::Select,
        'k',
        (Some(HelixAction::MoveLine(MoveDir1D::Previous)), None),
    ),
];

impl InputBindings<char, HelixStep> for HelixBindings {
    fn setup(&self, machine: &mut HelixMachine) {
        for &(mode, key, ref step) in BINDINGS {
            machine.add_mapping(mode, &[(EdgeRepeat::Once, EdgeEvent::Key(key))], step);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "hx")]
mod test {

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
        prelude::{TargetShape, ViewportContext},
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

        fn apply_motion(&mut self, target: &Option<EditTarget>) {
            let target = target.as_ref().expect("action should map to an EditTarget");
            let ectx = EditContextBuilder::default()
                .target_shape(Some(TargetShape::CharWise))
                .build();
            let ctx = &(self.gid, &self.vwctx, &ectx);
            self.ebuf
                .edit(&EditAction::Motion, target, ctx, &mut self.store)
                .unwrap();
        }

        fn leader(&mut self) -> Cursor {
            self.ebuf.get_leader(self.gid)
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
    #[case('h', HelixAction::MoveChar(MoveDir1D::Previous), Cursor::new(0, 1))]
    #[case('l', HelixAction::MoveChar(MoveDir1D::Next), Cursor::new(0, 3))]
    fn test_move_char(
        mut normal_machine: HelixMachine,
        #[case] key: char,
        #[case] expected_action: HelixAction,
        #[case] end: Cursor,
    ) {
        normal_machine.input_key(key);
        let (action, _) = normal_machine.pop().unwrap();
        assert_eq!(action, expected_action);

        let target = action.to_edit_target(Count::Exact(1));
        let mut tb = TestBuf::new("hello\n", Cursor::new(0, 2));
        tb.apply_motion(&target);
        assert_eq!(tb.leader(), end);
    }

    #[rstest]
    #[case('j', HelixAction::MoveLine(MoveDir1D::Next), Cursor::new(2, 2))]
    #[case('k', HelixAction::MoveLine(MoveDir1D::Previous), Cursor::new(0, 2))]
    fn test_move_line(
        mut normal_machine: HelixMachine,
        #[case] key: char,
        #[case] expected_action: HelixAction,
        #[case] end: Cursor,
    ) {
        normal_machine.input_key(key);
        let (action, _) = normal_machine.pop().unwrap();
        assert_eq!(action, expected_action);

        let target = action.to_edit_target(Count::Exact(1));
        let mut tb = TestBuf::new("hello\nworld\nfoo\n", Cursor::new(1, 2));
        tb.apply_motion(&target);
        assert_eq!(tb.leader(), end);
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
        let target = action.to_edit_target(Count::Exact(1));

        let mut tb = TestBuf::new("hello\n", Cursor::new(0, 2));
        tb.apply_motion(&target);

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
            let target = action.to_edit_target(Count::Exact(1));
            tb.apply_motion(&target);

            let sel = tb.selection().unwrap();
            assert_eq!(sel.0, start, "anchor should stay fixed");
            assert_eq!(sel.1, *expected_head, "head should have moved");
        }
    }
}
