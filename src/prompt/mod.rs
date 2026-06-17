mod base;
mod default;

#[cfg(feature = "helix")]
pub use base::PromptHelixMode;
pub use base::{
    Prompt, PromptEditMode, PromptEditModeDiscriminants, PromptHistorySearch,
    PromptHistorySearchStatus, PromptViMode,
};

pub use default::{DefaultPrompt, DefaultPromptSegment};
