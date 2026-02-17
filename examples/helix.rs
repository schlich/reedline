// Helix mode interactive tutorial and sandbox
//
// Interactive tutorial:
//   cargo run --example helix --features hx
//
// Sandbox mode:
//   cargo run --example helix --features hx -- --sandbox

#[cfg(feature = "hx")]
use reedline::{
    EditCommand, Helix, Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal,
};
#[cfg(feature = "hx")]
use std::borrow::Cow;
#[cfg(feature = "hx")]
use std::env;
#[cfg(feature = "hx")]
use std::io;

#[cfg(feature = "hx")]
#[derive(Clone, Copy)]
enum PromptStyle {
    Tutorial,
    Minimal,
}

#[cfg(feature = "hx")]
#[derive(Clone, Copy, PartialEq, Eq)]
enum TutorialStage {
    BasicEntry,
    NavigationAppend,
    Completed,
}

#[cfg(feature = "hx")]
enum SubmissionOutcome {
    Retry,
    Continue,
    Completed,
}

#[cfg(feature = "hx")]
struct HelixPrompt {
    style: PromptStyle,
}

#[cfg(feature = "hx")]
impl HelixPrompt {
    fn new(style: PromptStyle) -> Self {
        Self { style }
    }

    fn set_style(&mut self, style: PromptStyle) {
        self.style = style;
    }
}

#[cfg(feature = "hx")]
impl Prompt for HelixPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<'_, str> {
        let mode = match edit_mode {
            PromptEditMode::Custom(s) => s,
            _ => "HX_INSERT".to_string(),
        };

        match (self.style, mode.as_str()) {
            (PromptStyle::Tutorial, "HX_NORMAL") => Cow::Borrowed("[ NORMAL ] 〉"),
            (PromptStyle::Tutorial, "HX_INSERT") => Cow::Borrowed("[ INSERT ] : "),
            (PromptStyle::Tutorial, "HX_SELECT") => Cow::Borrowed("[ SELECT ] » "),
            (PromptStyle::Minimal, "HX_NORMAL") => Cow::Borrowed("〉"),
            (PromptStyle::Minimal, "HX_INSERT") => Cow::Borrowed(": "),
            (PromptStyle::Minimal, "HX_SELECT") => Cow::Borrowed("» "),
            _ => Cow::Borrowed("> "),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("::: ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            reedline::PromptHistorySearchStatus::Passing => "",
            reedline::PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }
}

#[cfg(feature = "hx")]
struct TutorialGuide {
    stage: TutorialStage,
}

#[cfg(feature = "hx")]
impl TutorialGuide {
    fn new() -> Self {
        Self {
            stage: TutorialStage::BasicEntry,
        }
    }

    fn handle_submission(&mut self, buffer: &str) -> SubmissionOutcome {
        match self.stage {
            TutorialStage::BasicEntry => {
                if buffer.trim() == "hello world" {
                    println!("\n🎉 Stage 1 Complete! 🎉");
                    println!("You entered INSERT mode and submitted text.");
                    println!("Next: a short NORMAL mode navigation + append exercise.\n");
                    self.stage = TutorialStage::NavigationAppend;
                    self.print_current_stage_instructions();
                    SubmissionOutcome::Continue
                } else {
                    println!("Not quite right. Expected: hello world");
                    println!("Try the stage again and submit.\n");
                    SubmissionOutcome::Retry
                }
            }
            TutorialStage::NavigationAppend => {
                if buffer.trim() == "hello world!" {
                    println!("\n🌟 Stage 2 Complete! 🌟");
                    println!("You used NORMAL mode motion + append to add punctuation.");
                    self.stage = TutorialStage::Completed;
                    SubmissionOutcome::Completed
                } else {
                    println!("Stage 2 not complete. Expected: hello world!");
                    println!("Hint: from NORMAL mode at end, press 'h', then 'a', type '!', Enter.\n");
                    SubmissionOutcome::Retry
                }
            }
            TutorialStage::Completed => SubmissionOutcome::Completed,
        }
    }

    fn stage(&self) -> TutorialStage {
        self.stage
    }

    fn print_current_stage_instructions(&self) {
        match self.stage {
            TutorialStage::BasicEntry => {
                println!("\n╭─ Stage 1: Insert Basics ─────────────────────────────────╮");
                println!("│  1. Press 'i' to enter INSERT mode                       │");
                println!("│  2. Type: hello world                                    │");
                println!("│  3. Press Enter to submit                                │");
                println!("╰──────────────────────────────────────────────────────────╯");
                println!("💡 Goal: submit exactly 'hello world'\n");
            }
            TutorialStage::NavigationAppend => {
                println!("\n╭─ Stage 2: Normal Navigation + Append ───────────────────╮");
                println!("│  Start buffer: hello world                              │");
                println!("│  1. Press Esc to ensure NORMAL mode                      │");
                println!("│  2. Press 'h'                                            │");
                println!("│  3. Press 'a', then type: !                              │");
                println!("│  4. Press Enter to submit                                │");
                println!("╰──────────────────────────────────────────────────────────╯");
                println!("💡 Goal: submit exactly 'hello world!'\n");
            }
            TutorialStage::Completed => {}
        }
    }
}

#[cfg(feature = "hx")]
fn preload_stage_two_buffer(line_editor: &mut Reedline) {
    line_editor.run_edit_commands(&[EditCommand::Clear]);
    line_editor.run_edit_commands(&[EditCommand::InsertString("hello world".to_string())]);
    line_editor.run_edit_commands(&[EditCommand::MoveToEnd { select: false }]);
}

#[cfg(feature = "hx")]
fn main() -> io::Result<()> {
    let sandbox_requested = env::args().skip(1).any(|arg| arg == "--sandbox");

    if sandbox_requested {
        println!("Helix Mode Sandbox");
        println!("==================");
        println!("Prompt: 〉(normal)  :(insert)  »(select)");
        println!("Keys available in this demo: i, a, v, h, j, k, l, w, Esc");
        println!("Exit: Ctrl+C or Ctrl+D\n");
    } else {
        println!("Helix Mode Interactive Tutorial");
        println!("================================\n");
        println!("This tutorial is scoped to currently implemented branch behavior.\n");
    }

    let mut line_editor = Reedline::create().with_edit_mode(Box::new(Helix::default()));
    let mut prompt = HelixPrompt::new(if sandbox_requested {
        PromptStyle::Minimal
    } else {
        PromptStyle::Tutorial
    });
    let mut guide = if sandbox_requested {
        None
    } else {
        Some(TutorialGuide::new())
    };
    let mut sandbox_active = sandbox_requested;

    if let Some(guide_ref) = guide.as_ref() {
        guide_ref.print_current_stage_instructions();
    }

    loop {
        let sig = line_editor.read_line(&prompt)?;

        match sig {
            Signal::Success(buffer) => {
                if let Some(guide_ref) = guide.as_mut() {
                    match guide_ref.handle_submission(&buffer) {
                        SubmissionOutcome::Retry => {}
                        SubmissionOutcome::Continue => {
                            if guide_ref.stage() == TutorialStage::NavigationAppend {
                                preload_stage_two_buffer(&mut line_editor);
                            }
                            continue;
                        }
                        SubmissionOutcome::Completed => {
                            println!("Tutorial complete. Continuing in sandbox mode.\n");
                            prompt.set_style(PromptStyle::Minimal);
                            guide = None;
                            sandbox_active = true;
                            continue;
                        }
                    }
                } else if sandbox_active {
                    println!("{buffer}");
                }
            }
            Signal::CtrlC | Signal::CtrlD => {
                println!("\nGoodbye! 👋");
                break Ok(());
            }
        }
    }
}

#[cfg(not(feature = "hx"))]
fn main() {
    println!("Re-run with `--features hx` to launch the Helix demo.");
}