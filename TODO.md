# TODOs

# v0.0.0-beta.0
This is a partial rewrite of Tempo aimed at establishing long-term stability for the project.

- [x] switch to anyhow
- [ ] establish concrete data model
  - [ ] importantly including how schema changes will work
- [ ] set up frontend sql access
  - just use tauri sql plugin for now
- [ ] switch to sqlx
  - if we're using tauri sql plugin might be best to switch to sqlx as well
  - sqlx advises against combining with rusqlite due to possible linking issues
  - might be a little awkward with async stuff but not a big deal
- [ ] ditch lifetime state management model
  - could add system for prioritizing content in view on rescan?
    - basic register/unregister for specific items in folders
  - reactive updates:
    - emit rowid or something?
- [ ] write tests throughout tempo
- [ ] remove sql from shared data
- [ ] rewrite frontend in svelte, focus on theming and actual good ui development from the start
  - this is going to take a long time, but is probably worth it
- [ ] enforce single instance, probably just tauri plugin