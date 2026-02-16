use modalkit::prelude::{MoveDir1D, MoveType};
use modalkit::keybindings::{
    BindingMachine, EmptyKeyState, InputBindings, InputKey, ModalMachine, Mode, ModeKeys, EdgeEvent, EdgeRepeat
};

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
    Move(MoveType),
    #[default]
    NoOp,
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
            &(Some(HelixAction::Move(MoveType::Column(MoveDir1D::Previous, false))), None),
        );
    }
}

type HelixStep = (Option<HelixAction>, Option<HelixMode>);
pub type HelixMachine = ModalMachine<char, HelixStep>;


#[cfg(test)]
#[cfg(feature = "hx")]
mod test {

    use super::*;

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
        let action = machine.pop();
        assert_eq!(
            action,
            Some((HelixAction::Move(MoveType::Column(MoveDir1D::Previous, false)), EmptyKeyState::default()))
        )
    }

}
