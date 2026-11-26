//! --- agentic ask --- Ask… on the Agentic feed tab: the reader asks for an
//! edition in their own words ("could you summarize the latest from Ada and
//! Rue on tides and boats"), and Burrow's AI writes it
//! (`agentic_writer::Asked`).
//!
//! - The blygs the request names are found as it's typed
//!   (`agentic_write::named_sources`, over the subscriptions: the sidebar's
//!   names, a bracketed tag in one, or a host), and the sheet says whose
//!   posts will be read. None named: the latest posts from everyone.
//! - ⏎ closes the sheet and writes the edition; the tab's line says
//!   "Writing an edition with …" as it does for ⌥⌘E. The posts are the
//!   latest from those blygs, read or not; the topics and shape are left to
//!   the model, which is told to follow the request over the blyg's brief.
//! - Ask… again starts from the last request, to adjust it.

use blyg_core::SubId;
use blyg_core::agentic_write::named_sources;
use gpui_kit::base::input::{Escape, InputEvent, InputState};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use super::RSheet;
use super::agentic_writer::Asked;
use crate::app::MainView;

/// The Ask sheet: the request, and the blygs it names so far.
pub struct AskSheet {
    pub input: Entity<InputState>,
    pub sources: Vec<SubId>,
    pub error: Option<String>,
}

/// What the sheet says under the request: whose posts will be read.
pub fn from_words(names: &[String]) -> String {
    match names {
        [] => "No blyg named yet: the latest posts from every blyg you follow. Name blygs as \
               the sidebar names them to read only theirs."
            .into(),
        [one] => format!("From {one}: its latest posts, read or not."),
        [rest @ .., last] => format!(
            "From {} and {last}: their latest posts, read or not.",
            rest.join(", ")
        ),
    }
}

impl MainView {
    /// Ask… on the Agentic feed tab.
    pub(crate) fn open_agentic_ask_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let last = self.reading.agentic_writer.last_request.clone();
        let sources = named_sources(&last, &self.reading.subs);
        let input = cx.new(|cx| {
            let mut s = InputState::new(window, cx)
                .placeholder("e.g. the latest from Ada and Rue on tides and boats");
            s.set_value(last, window, cx);
            s
        });
        let sub = cx.subscribe_in(&input, window, |this, _, ev, window, cx| match ev {
            InputEvent::PressEnter { .. } => this.commit_agentic_ask_sheet(window, cx),
            InputEvent::Change => this.agentic_ask_changed(cx),
            _ => {}
        });
        self._subs.push(sub);
        input.update(cx, |s, cx| {
            s.focus(window, cx);
            s.select_all(window, cx);
        });
        self.open_reading_sheet(
            RSheet::AgenticAsk(Box::new(AskSheet {
                input,
                sources,
                error: None,
            })),
            cx,
        );
    }

    /// Typing: find the blygs the request names (the subscriptions are a
    /// short list, so this is cheap on every key).
    fn agentic_ask_changed(&mut self, cx: &mut Context<Self>) {
        let Some(RSheet::AgenticAsk(s)) = self.reading.sheet.as_mut() else {
            return;
        };
        let text = s.input.read(cx).value().to_string();
        let sources = named_sources(&text, &self.reading.subs);
        let cleared = !text.trim().is_empty() && s.error.take().is_some();
        if sources != s.sources || cleared {
            s.sources = sources;
            cx.notify();
        }
    }

    /// ⏎ in the Ask sheet: write the edition to the request.
    pub(crate) fn commit_agentic_ask_sheet(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(RSheet::AgenticAsk(s)) = self.reading.sheet.as_mut() else {
            return;
        };
        let text = s.input.read(cx).value().trim().to_string();
        if text.is_empty() {
            s.error = Some("Say what the edition should cover: blygs, topics, or both.".into());
            cx.notify();
            return;
        }
        let sources = named_sources(&text, &self.reading.subs);
        self.close_reading_sheet(window, cx);
        self.agentic_write_asked(Asked { text, sources }, cx);
    }

    pub(super) fn render_agentic_ask_sheet(
        &self,
        s: &AskSheet,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let p = self.palette.on_page();
        let names: Vec<String> = s
            .sources
            .iter()
            .filter_map(|id| self.reading.subs.iter().find(|x| &x.id == id))
            .map(|x| x.title.clone())
            .collect();
        div()
            .capture_action(cx.listener(|this, _: &Escape, window, cx| {
                cx.stop_propagation();
                this.close_reading_sheet(window, cx);
            }))
            .child(self.sheet_heading("Ask for an Edition"))
            .child(div().mb(px(8.)).text_color(p.muted).child(
                "Say what this edition should cover. Burrow's AI writes it from the latest posts \
                 of the blygs you name, on the topics you name, and it replaces the tab's \
                 edition.",
            ))
            .child(self.input_box(
                gpui_kit::base::input::Input::new(&s.input).into_any_element(),
                s.error.is_some(),
            ))
            .when_some(s.error.clone(), |d, e| {
                d.child(div().mt(px(6.)).text_color(p.over).child(e))
            })
            .child(
                div()
                    .id("agentic-ask-from")
                    .debug_selector(|| "agentic-ask-from".into())
                    .mt(px(6.))
                    .text_color(p.muted)
                    .child(from_words(&names)),
            )
            .child(self.keys_row(vec![
                self.key_hint("⏎", "write"),
                self.key_hint("esc", "cancel"),
            ]))
            .into_any_element()
    }
}
