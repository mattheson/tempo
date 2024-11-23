various design notes, will probably delete these

# data model
how schema changes will work still seems unclear to me
automerge complicates things, removing it could be an option.
automerge could be used in the future for realtime features?
i have no need to store complete edit history of documents in tempo; tempo's chat log design serves as the history
furthermore i intend to make it so that notes can't be completely deleted, they probably can be hidden though

take comments out of notes directly? like

```rust
enum Note {
    Comment(Ref)
    AbletonProject(Info)
    Audio(Info)
}
```

i like the idea of making notes into smallest user-created data type
we want to notify when something new is created by another user, with new design above we'd just notify when new notes are indexed. pretty simple
makes the most sense for notes to be mutable by sending user, don't see much of a purpose for other users editing notes
this simplifies things a lot, it would just make tempo into a big DAG

there are some parts of folders which might can be edited by multiple users, for now this is just
- usernames
- folder name
- channel names
- visibility

say a user two users edit a name of a channel. tempo could ensure the new name always applies on the individual editors' end, but this behavior would be unexpected, there needs to be some conflict resolution
simplest solution is to compare the uuids of all the conflicting editors. result should be arbitrary

new folder layout
- folder
  - `tempo-session`
    - `[uuid]`
      - `meta`: this user's metadata
      - `session`: session metadata
      - `[ulid]`: a note
    - `files`
      - `[sha256]`: file with file header

file (n bytes) header layout:
[4 bytes] (u32, header schema number)
[8 bytes] (x: u64, total size of header)
[x - 12 bytes] header (thinking zstd compressed json)
[n - x bytes] actual file

for now i think the impact of file streaming to copy referenced files should not be a big deal, will need to make own tauri streaming protocol for audio player

# sql
why sql? for notifications and for granular loading of notes to frontend.
need to have some known state of a folder for notifications
think it's best for sql schema to stay dead similar between versions and avoid changes as much as possible
can add migrations if needed?

generally for tempo to become serious i expect sql will be necessary

misc
schema number | uuid | frontend kv store

sessions
id | provider | provider id | name

notes
id | latest (id of latest edit) | folder id | provider note id | creator uuid | channel note id (optional) | new (bool)

ancestry
parent id | child id

plugin_ids
id | format-specific id | name | vendor

do we need sql plugin?
probably not, operations are pretty simple and well defined
just use a static for all connections

# beta mvp
- cover all of alpha
  - creating notes, scanning projects, creating copies
  - comments are shifted towards text replies to notes
- editing of notes
- hiding of notes