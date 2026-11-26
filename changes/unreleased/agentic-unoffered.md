section: Fixed

- **Agentic feed: the key answers on a studio without editions.** On a blyg
  whose studio is older than 0.69, ⌥⌘E (Ctrl+Shift+J on Windows) used to
  do nothing visible: the writer's line, where its answer goes, was hidden
  there. Now the line says the studio doesn't offer editions yet, and the
  empty tab no longer promises that the key or Settings › Agentic feed will
  write one until the studio is updated. When the studio has been updated
  since the tab last looked, the key writes the edition and the tab catches
  up. (`crates/blyg-app/src/reading/agentic_writer.rs`,
  `crates/blyg-app/src/reading/agentic.rs`)
