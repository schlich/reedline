use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::{
    enums::{ReedlineEvent, ReedlineRawEvent},
    PromptEditMode,
};

use super::EditMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HelixMode {
    Normal,
    Insert,
}

pub struct Helix {
    mode: HelixMode,
    cache: Vec<char>,
}

impl Default for Helix {
    fn default() -> Self {
        Self {
            mode: HelixMode::Insert,
            cache: Vec::new(),
        }
    }
}

impl EditMode for Helix {
    fn parse_event(&mut self, event: ReedlineRawEvent) -> ReedlineEvent {
        match event.into() {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (self.mode, modifiers, code) {
                (HelixMode::Insert, KeyModifiers::NONE, KeyCode::Esc) => {
                    self.cache.clear();
                    self.mode = HelixMode::Normal;
                    ReedlineEvent::Multiple(vec![ReedlineEvent::Esc, ReedlineEvent::Repaint])
                }
                _ => ReedlineEvent::None,
            },
            _ => ReedlineEvent::None,
        }
    }

    fn edit_mode(&self) -> PromptEditMode {
        PromptEditMode::Custom("hx".to_string())
    }
}

#[cfg(test)]
#[cfg(feature = "hx")]
mod test {
    use super::*;

    #[test]
    fn test_esc_leads_to_normal_mode() {
        let mut hx = Helix::default();
        let esc =
            ReedlineRawEvent::try_from(Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)))
                .unwrap();
        let result = hx.parse_event(esc);

        assert_eq!(
            result,
            ReedlineEvent::Multiple(vec![ReedlineEvent::Esc, ReedlineEvent::Repaint])
        );
        assert_eq!(hx.mode, HelixMode::Normal);
    }
}
