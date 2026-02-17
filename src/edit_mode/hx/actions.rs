use modalkit::prelude::EditTarget;

/// Actions produced by the experimental Helix edit mode key machine.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum HelixAction {
    Type(char),
    Motion(EditTarget),
    #[default]
    NoOp,
}

impl HelixAction {
    pub(super) fn motion_target(&self) -> Option<&EditTarget> {
        match self {
            HelixAction::Motion(target) => Some(target),
            _ => None,
        }
    }
}