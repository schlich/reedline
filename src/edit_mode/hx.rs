use modalkit::{
    editing::context::EditContext,
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
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum HelixAction {
    Type(char),
    MoveChar(MoveDir1D),
    #[default]
    NoOp,
}

impl TryFrom<HelixAction> for MoveType {
    type Error = ();

    fn try_from(action: HelixAction) -> Result<Self, ()> {
        match action {
            HelixAction::MoveChar(dir) => Ok(MoveType::Column(dir, false)),
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

#[derive(Default)]
struct HelixBindings;

impl Mode<HelixAction, EmptyKeyState> for HelixMode {}

impl<K: InputKey> ModeKeys<K, HelixAction, EmptyKeyState> for HelixMode {
    fn unmapped(&self, key: &K, _: &mut EmptyKeyState) -> (Vec<HelixAction>, Option<HelixMode>) {
        match self {
            HelixMode::Normal => (vec![], None),
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
];

impl InputBindings<char, HelixStep> for HelixBindings {
    fn setup(&self, machine: &mut HelixMachine) {
        for &(mode, key, ref step) in BINDINGS {
            machine.add_mapping(mode, &[(EdgeRepeat::Once, EdgeEvent::Key(key))], step);
        }
    }
}

type HelixStep = (Option<HelixAction>, Option<HelixMode>);
pub type HelixMachine = ModalMachine<char, HelixStep>;

#[cfg(test)]
#[cfg(feature = "hx")]
mod test {

    use super::*;
    use modalkit::{
        actions::{EditAction, EditorActions},
        editing::{
            application::EmptyInfo,
            buffer::{CursorGroupId, EditBuffer},
            context::EditContextBuilder,
            cursor::{Cursor, CursorGroup, CursorState},
            store::Store,
        },
        env::vim::VimState,
        prelude::{TargetShape, ViewportContext},
    };

    use rstest::*;

    fn mkbuf(
        s: &str,
        start: Cursor,
    ) -> (
        EditBuffer<EmptyInfo>,
        CursorGroupId,
        ViewportContext<Cursor>,
        VimState,
        Store<EmptyInfo>,
    ) {
        let mut buf = EditBuffer::new("".to_string());
        let gid = buf.create_group();
        let vwctx = ViewportContext::default();
        let vctx = VimState::default();
        let store = Store::default();

        buf.set_text(s);

        let leader = CursorState::Selection(start.clone(), start, TargetShape::CharWise);
        buf.set_group(gid, CursorGroup::new(leader, vec![]));

        (buf, gid, vwctx, vctx, store)
    }

    fn helix_edit_context() -> EditContext {
        EditContextBuilder::default()
            .target_shape(Some(TargetShape::CharWise))
            .build()
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

        let (mut ebuf, gid, vwctx, _vctx, mut store) = mkbuf("hello\n", Cursor::new(0, 2));
        let ectx = helix_edit_context();
        let ctx = &(gid, &vwctx, &ectx);
        ebuf.edit(&EditAction::Motion, &target, ctx, &mut store)
            .unwrap();
        assert_eq!(ebuf.get_leader(gid), end);
    }

    #[test]
    fn test_cursor_always_has_selection() {
        let (mut ebuf, gid, _, _, _) = mkbuf("hello\n", Cursor::new(0, 2));

        let sel = ebuf.get_leader_selection(gid);
        assert_eq!(
            sel,
            Some((Cursor::new(0, 2), Cursor::new(0, 2), TargetShape::CharWise))
        );
    }

    #[rstest]
    fn test_selection_survives_motion(mut normal_machine: HelixMachine) {
        normal_machine.input_key('l');
        let (action, _) = normal_machine.pop().unwrap();
        let target = action
            .to_edit_target(Count::Exact(1))
            .expect("action should map to an EditTarget");

        let (mut ebuf, gid, vwctx, _vctx, mut store) = mkbuf("hello\n", Cursor::new(0, 2));
        let ectx = helix_edit_context();
        let ctx = &(gid, &vwctx, &ectx);
        ebuf.edit(&EditAction::Motion, &target, ctx, &mut store)
            .unwrap();

        let sel = ebuf.get_leader_selection(gid).unwrap();
        assert_eq!(sel.0, Cursor::new(0, 2), "anchor should stay at start");
        assert_eq!(sel.1, Cursor::new(0, 3), "head should have moved");
        assert_eq!(sel.2, TargetShape::CharWise, "shape should remain CharWise");
    }
}
