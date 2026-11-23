//! --- read responses --- Headless GPUI tests on the zero-latency
//! FakeBackend: a response opens beside the Responses list in the same
//! reading pane as the Reader (⏎ or a click), j/k carry the pane along, esc
//! closes it before leaving the screen, and a response that isn't a blyg
//! post still opens in the browser.

use std::sync::Arc;

use blyg_core::config::MemoryTokenStore;
use blyg_core::{Backend, ConfigStore};
use gpui_kit::{Entity, TestAppContext, VisualTestContext};

use super::View;
use crate::app::MainView;
use crate::fake::reading_seed::*;
use crate::fake::{FakeBackend, Timing};
use crate::prefs::Prefs;
use crate::test_wait::settle;

const CONNECTED: &str = "# test config\nblyg-url = https://blyg.example.com\n";

fn setup(cx: &mut TestAppContext) -> (Entity<MainView>, Arc<FakeBackend>, &mut VisualTestContext) {
    let prefs = Prefs::from_config(ConfigStore::in_memory(CONNECTED).config());
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::app::bind_keys(cx);
        crate::settings::init(
            ConfigStore::in_memory(CONNECTED),
            Arc::new(MemoryTokenStore::default()),
            None,
            cx,
        );
    });
    let fake = Arc::new(FakeBackend::with_timing(Timing::instant()).without_media_cache());
    // The sample responses' posts, as their blygs' public item documents
    // serve them (none of them is held in the reading list).
    fake.add_public_post(RUE, "01K2Mm1", "Rue's reply: friction is a teacher.");
    fake.add_public_post(ADA, "01K2Mm2", "Ada quotes it: the step is the lesson.");
    fake.add_public_post(LIN, "01K2Mm3", "Lin forks it into a harbour story.");
    let backend: Arc<dyn Backend> = fake.clone();
    let f2 = fake.clone();
    let (view, cx) = cx.add_window_view(move |window, cx| {
        MainView::new(
            backend,
            Some(f2),
            prefs,
            std::time::Instant::now(),
            window,
            cx,
        )
    });
    cx.run_until_parked();
    cx.simulate_keystrokes(&crate::keymap::keys("cmd-shift-m"));
    settle(cx);
    view.read_with(cx, |v, _| assert_eq!(v.reading.view, View::Mentions));
    (view, fake, cx)
}

/// The id of the post open in the reading pane, if any.
fn opened(view: &Entity<MainView>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |v, _| {
        v.reading
            .opened
            .as_ref()
            .map(|o| o.item.remote_id.to_string())
    })
}

/// The text the reading pane's web view would show.
fn shown_text(view: &Entity<MainView>, cx: &mut VisualTestContext) -> Option<String> {
    view.read_with(cx, |v, _| v.reader_doc().map(|d| format!("{:?}", d.body)))
}

#[gpui_kit::test]
fn enter_reads_a_response_beside_the_list(cx: &mut TestAppContext) {
    let (view, fake, cx) = setup(cx);
    assert_eq!(opened(&view, cx), None);
    assert_eq!(
        shown_text(&view, cx),
        None,
        "no pane before a response is read"
    );

    view.update(cx, |v, _| v.reading.mention_sel = Some("m1".into()));
    cx.simulate_keystrokes("enter");
    settle(cx);
    view.read_with(cx, |v, _| {
        assert_eq!(v.reading.view, View::Mentions, "still on Responses");
        assert_eq!(v.reading.mention_reading.as_deref(), Some("m1"));
    });
    assert_eq!(opened(&view, cx).as_deref(), Some("01K2Mm1"));
    let text = shown_text(&view, cx).expect("the reading pane shows it");
    assert!(text.contains("friction is a teacher"), "{text}");
    // Fetched from its blyg's public item document, without a token.
    assert!(
        fake.public_fetches()
            .iter()
            .any(|u| u.ends_with("items/01K2Mm1.json")),
        "{:?}",
        fake.public_fetches()
    );
}

#[gpui_kit::test]
fn j_and_k_carry_the_reading_pane_along(cx: &mut TestAppContext) {
    let (view, _, cx) = setup(cx);
    let order: Vec<String> = view.read_with(cx, |v, _| {
        v.mention_order().into_iter().map(|r| r.id).collect()
    });
    let first = order[0].clone();
    view.update(cx, |v, _| v.reading.mention_sel = Some(first.clone()));
    cx.simulate_keystrokes("enter");
    settle(cx);
    let before = opened(&view, cx).expect("open");
    // The next response with a blyg post of its own (link notices count too).
    cx.simulate_keystrokes("j");
    settle(cx);
    let (sel, reading) = view.read_with(cx, |v, _| {
        (
            v.reading.mention_sel.clone(),
            v.reading.mention_reading.clone(),
        )
    });
    assert_ne!(sel.as_deref(), Some(first.as_str()));
    assert_eq!(reading, sel, "the pane follows the cursor");
    assert_ne!(opened(&view, cx), Some(before));
}

#[gpui_kit::test]
fn esc_closes_the_pane_then_leaves(cx: &mut TestAppContext) {
    let (view, _, cx) = setup(cx);
    view.update(cx, |v, _| v.reading.mention_sel = Some("m1".into()));
    cx.simulate_keystrokes("enter");
    settle(cx);
    assert!(shown_text(&view, cx).is_some());

    cx.simulate_keystrokes("escape");
    settle(cx);
    view.read_with(cx, |v, _| {
        assert_eq!(v.reading.view, View::Mentions, "esc closes the pane first");
        assert_eq!(v.reading.mention_reading, None);
        assert_eq!(
            v.reading.mention_sel.as_deref(),
            Some("m1"),
            "the cursor stays"
        );
    });
    assert_eq!(shown_text(&view, cx), None, "the web view is put away");

    cx.simulate_keystrokes("escape");
    settle(cx);
    view.read_with(cx, |v, _| assert_ne!(v.reading.view, View::Mentions));
}

#[gpui_kit::test]
fn a_response_that_isnt_a_blyg_post_opens_in_the_browser(cx: &mut TestAppContext) {
    let (view, fake, cx) = setup(cx);
    // A plain web page that replied: no item id to read here.
    fake.set_mention_source("m1", "https://web.example.com/a-reply", None);
    view.update_in(cx, |v, _, cx| v.load_mentions(cx));
    settle(cx);
    view.update(cx, |v, _| v.reading.mention_sel = Some("m1".into()));
    cx.simulate_keystrokes("enter");
    settle(cx);
    view.read_with(cx, |v, _| {
        assert_eq!(v.reading.mention_reading, None);
        assert!(
            v.toast.as_ref().is_some_and(|t| t.text.contains("browser")),
            "says where it went"
        );
    });
    assert_eq!(shown_text(&view, cx), None);
}

#[gpui_kit::test]
fn the_header_names_the_reading_key(cx: &mut TestAppContext) {
    let (view, _, cx) = setup(cx);
    let hint = view.read_with(cx, |v, _| v.mentions_hint());
    assert!(hint.contains("⏎ read"), "{hint}");
    assert!(hint.contains("o open ↗"), "{hint}");
}
