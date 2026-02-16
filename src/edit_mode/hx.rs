use keybindings::{
    BindingMachine, EmptyKeyState, InputBindings, InputKey, ModalMachine, Mode, ModeKeys,
};

const ESC: char = '\u{1B}';

#[derive(Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
enum HelixMode {
    #[default]
    Insert,
    Normal,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
enum HelixAction {
    Type(char),
    #[default]
    NoOp,
    Quit,
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
        use keybindings::EdgeEvent::Key;
        use keybindings::EdgeRepeat::Once;

        machine.add_mapping(
            HelixMode::Insert,
            &[(Once, Key(ESC))],
            &(None, Some(HelixMode::Normal)),
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
}
