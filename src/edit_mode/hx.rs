use modalkit::keybindings::{
    BindingMachine, EmptyKeyState, InputBindings, InputKey, ModalMachine, Mode, ModeKeys, EdgeEvent, EdgeRepeat
};
use modalkit::prelude::{MoveDir1D, MoveType};

const ESC:char = '\u{1B}';

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
enum HelixMode {
    #[default]
    Insert,
    Normal,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum HelixAction {
    Type(char),
    MoveCharLeft,
    #[default]
    NoOp,
}

impl HelixAction {
    fn to_move_type(&self) -> Option<MoveType> {
        match self {
            HelixAction::MoveCharLeft => Some(MoveType::Column(MoveDir1D::Previous, false)),
            _ => None,
        }
    }
}

#[derive(Default)]
struct HelixBindings {}

impl Mode<HelixAction, EmptyKeyState> for HelixMode {}

impl<K: InputKey> ModeKeys<K, HelixAction, EmptyKeyState> for HelixMode {
    fn unmapped(&self, key: &K, _: &mut EmptyKeyState) -> (Vec<HelixAction>, Option<HelixMode>) {
        match self {
            HelixMode::Normal => {
                return (vec![], None);
            }
            HelixMode::Insert => {
                if let Some(c) = key.get_char() {
                    return (vec![HelixAction::Type(c)], None);
                }

                return (vec![], None);
            }
        }
    }
}

impl InputBindings<char, HelixStep> for HelixBindings {
    fn setup(&self, machine: &mut HelixMachine) {

        machine.add_mapping(
            HelixMode::Insert,
            &[(EdgeRepeat::Once, EdgeEvent::Key(ESC))],
            &(None, Some(HelixMode::Normal)),
        );

        machine.add_mapping(
            HelixMode::Normal,
            &[(EdgeRepeat::Once, EdgeEvent::Key('h'))],
            &(Some(HelixAction::MoveCharLeft), None),
        );
    }
}

type HelixStep = (Option<HelixAction>, Option<HelixMode>);
pub type HelixMachine = ModalMachine<char, HelixStep>;


#[cfg(test)]
#[cfg(feature = "hx")]
mod test {

    use super::*;
    use modalkit::editing::application::EmptyInfo;
    use modalkit::editing::buffer::EditBuffer;
    use modalkit::editing::cursor::Cursor;
    use modalkit::editing::store::Store;
    use modalkit::prelude::{Count, EditTarget, ViewportContext};
    use modalkit::actions::{EditAction, EditorActions};
    use modalkit::env::vim::VimState;

    fn mkfivestr(
        s: &str,
    ) -> (EditBuffer<EmptyInfo>, modalkit::editing::buffer::CursorGroupId, ViewportContext<Cursor>, VimState, Store<EmptyInfo>)
    {
        let mut buf = EditBuffer::new("".to_string());
        let gid = buf.create_group();
        let vwctx = ViewportContext::default();
        let vctx = VimState::default();
        let store = Store::default();

        buf.set_text(s);

        (buf, gid, vwctx, vctx, store)
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

    #[test]
    fn test_move_char_left_action() {
        let mut machine = HelixMachine::from_bindings::<HelixBindings>();

        machine.input_key(ESC);
        assert_eq!(machine.mode(), HelixMode::Normal);
        let _ = machine.pop();

        machine.input_key('h');
        let (action, _) = machine.pop().unwrap();
        assert_eq!(action, HelixAction::MoveCharLeft);

        let mv = action.to_move_type().expect("MoveCharLeft should map to a MoveType");

        let (mut ebuf, gid, vwctx, vctx, mut store) = mkfivestr("hello\n");
        ebuf.set_leader(gid, Cursor::new(0, 3));
        assert_eq!(ebuf.get_leader(gid), Cursor::new(0, 3));

        let target = EditTarget::Motion(mv, Count::Exact(1));
        let ctx = &(gid, &vwctx, &modalkit::editing::context::EditContext::default());
        ebuf.edit(&EditAction::Motion, &target, ctx, &mut store).unwrap();
        assert_eq!(ebuf.get_leader(gid), Cursor::new(0, 2));
    }

}
