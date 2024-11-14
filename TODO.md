# TODOs

# v0.0.0-beta.0
This release is focused on improving internals along with lots of refactoring.

- [x] switch to anyhow
- [ ] set up frontend sql access
  - just use tauri sql plugin for now
- [ ] switch to sqlx
  - if we're using tauri sql plugin might be best to switch to sqlx as well
  - might be a little awkward with async stuff but not a big deal
- [ ] ditch lifetime state management model?
  - could add system for prioritizing content in view on rescan?
  - db emits updates to rows?, components register for updates
    - i.e. note changes, backend emits up
- [ ] write tests throughout tempo
- [ ] set up sql data model
- [ ] remove sql from shared data
- [ ] finalize folder structure changes
- [ ] rewrite frontend in svelte, focus on theming from the start