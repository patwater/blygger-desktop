section: Added

- **Agentic feed: Ask… for an edition in your own words.** Beside "Write
  an edition now" on the Agentic feed tab, Ask… opens a sheet for a request
  such as "could you summarize the latest from Ada and Rue on tides and
  boats". The blygs it names are found as you type (by the sidebar's name,
  a bracketed tag in it, or the blyg's address), and the sheet says whose
  posts it will read. ⏎ has Burrow's AI write the edition from their latest
  posts, read or not, taken from each blyg in turn, and the model is told to
  follow the request over your blyg's Agentic feed prompt. With no blyg
  named it reads the latest from everyone. Ask… again starts from the last
  request. (`crates/blyg-app/src/reading/agentic_ask.rs`,
  `blyg_core::agentic_write::named_sources` and `asked_candidates`,
  `blyg_ai::prompts::agentic_asked_prompt`)
