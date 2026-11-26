//! --- agentic writer --- The prompt Burrow's own AI writes the Agentic
//! feed's edition from: the blyg's brief (its Agentic feed prompt) when it
//! has one, else Burrow's; each post it's given, numbered, with its source,
//! title and opening words; JSON only.

use blyg_ai::prompts::{AGENTIC_DEFAULT_BRIEF, agentic_asked_prompt, agentic_edition_prompt};
use blyg_core::agentic_write::Candidate;
use blyg_core::{RemoteId, SubId};

fn cand(i: usize, title: Option<&str>) -> Candidate {
    Candidate {
        sub: SubId::from("sub-a"),
        remote_id: RemoteId::from(format!("r{i}").as_str()),
        version: 3,
        source: "Ada's harbour".into(),
        title: title.map(str::to_string),
        excerpt: format!("Invented words {i}."),
        at: "2026-11-22T09:00:00.000Z".into(),
        observed_at: "2026-11-22T09:00:00.000Z".into(),
    }
}

#[test]
fn the_prompt_numbers_each_post_and_asks_for_json_only() {
    let given = [cand(1, Some("Tide tables")), cand(2, None)];
    let p = agentic_edition_prompt(None, &given);
    // A newspaper: title, lede, a lead, sections, briefs, why each is there.
    for w in [
        "title",
        "lede",
        "\"lead\"",
        "\"briefs\"",
        "reason",
        "\"ref\"",
    ] {
        assert!(p.system.contains(w), "the system prompt names {w}");
    }
    assert!(p.system.contains("ONLY"), "JSON only");
    assert!(p.user.contains(AGENTIC_DEFAULT_BRIEF), "Burrow's brief");
    assert!(
        p.user.contains("[1] Ada's harbour · Tide tables"),
        "{}",
        p.user
    );
    assert!(p.user.contains("Invented words 1."));
    assert!(
        p.user.contains("[2] Ada's harbour · untitled"),
        "{}",
        p.user
    );
    assert!(
        !p.user.contains("r1") && !p.user.contains("sub-a"),
        "ids stay with Burrow: the model names posts by number"
    );
    // The blyg's own brief replaces Burrow's.
    let p = agentic_edition_prompt(Some("Only boats, please."), &given);
    assert!(p.user.contains("Only boats, please."));
    assert!(!p.user.contains(AGENTIC_DEFAULT_BRIEF));
    // A blank brief is no brief.
    let p = agentic_edition_prompt(Some("  "), &given);
    assert!(p.user.contains(AGENTIC_DEFAULT_BRIEF));
}

/// --- agentic ask --- An edition to the reader's own request: the
/// request is given with the brief, and wins where they differ; the posts
/// are numbered as before; a very long request is cut.
#[test]
fn a_request_is_given_and_followed_over_the_brief() {
    let given = [cand(1, Some("Tide tables")), cand(2, None)];
    let ask = "Could you please summarize the latest from Ada's harbour on tides and boats?";
    let p = agentic_asked_prompt(Some("Only boats, please."), &given, ask);
    assert!(p.user.contains(ask), "{}", p.user);
    assert!(
        p.user.contains("Only boats, please."),
        "the brief still sets the tone"
    );
    assert!(
        p.user.contains("follow the request"),
        "the request wins: {}",
        p.user
    );
    assert!(p.user.contains("[1] Ada's harbour · Tide tables"));
    assert_eq!(p.system, agentic_edition_prompt(None, &given).system);

    let long = "boats ".repeat(400);
    let p = agentic_asked_prompt(None, &given, &long);
    assert!(p.user.contains(AGENTIC_DEFAULT_BRIEF));
    let max = blyg_core::agentic_write::REQUEST_MAX;
    assert_eq!(max, 1_000);
    assert!(
        p.user.matches("boats").count() <= max / "boats ".len() + 1,
        "the request is cut to REQUEST_MAX"
    );
}
