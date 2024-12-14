# Design
This document covers Tempo's design and internals.

## Features

### File References
- automated "Collect All and Save"
  - automatic copying of referenced files in project files
  - adjusts file references within project files to point into `Files` directory

### Plugin Synchronization
- plugin synchronization checks
  - shared plugin data between users
  - scanning of project file to check for plugins collaborators are missing

I still feel like Tempo doesn't completely address the problem of plugin synchronization. It would be preferable to *prevent* users from adding incompatible plugins into projects altogether.

Tempo could monitor a project file. When the file is changed, it could be scanned to see if any plugins were added. If Tempo finds a plugin that other users are missing, it could show an alert to the user.

## Architecture
Since this application uses Tauri, Tempo consists of a WebView frontend and a Rust backend.

Generally the backend should do most data processing/data-intensive stuff. The frontend should just show stuff. Pretty standard.

We use a Cargo workspace with business logic split into various crates for better incremental compliation.

## Data Directory
For simple setup and cleanup, Tempo's data directory only contains the `tempo.sqlite` database, possibly with log files in the future.

## State Management
State management can be tricky with Tauri. You have to pass data between the frontend and backend using Tauri's command/event systems.

*How do we read data on the frontend?*

The simplest solution I've found is to write **everything** into a SQLite database, and give the frontend readonly access to this database using Tauri's SQL plugin. This seems like the most flexible way for the frontend to read data. Directly using SQL on the frontend allows us to avoid writing a bunch of commands/events to pass data to the frontend.

## DAW Support
Tempo only supports Ableton, but I'd like it to support other DAWs in the future.

# Other notes
## integration with medium
Many collaborative/version management tools are deeply integrated with the medium they're managing.
For example, with Git, there's an expectation that users are writing code, which allows Git to provide lots of text/code-specific features (e.g. merge conflict markers, generating patches)

Unfortunately we don't have similar deep integration with project files from DAWs, since the format of these files is usually never publicly documented. This means Tempo can't provide complex features like Git (e.g. diffing project files), we have to treat project files as being mostly opaque. This will remain the case until tools exist for reliably parsing project file contents, or until DAWs start supporting something like DAWproject.

There is one exception to this: Tempo does parse project files to read *file references and plugin references* since these references often break, and they're a very small subset of the contents of project files, which makes them much easier to parse/reverse engineer. However, Tempo doesn't handle reading/writing the actual composition (track/sample layout, MIDI, etc.) of project files.

## version management expectations
Version management systems are usually focused on managing divergent versions of a medium combined with an expectation that users will converge divergent versions (merging).

Tempo has a different approach compared to systems like Git. Tempo is much more focused on just the **organization** aspect of different versions, primarily because it's difficult to integrate deeply with project files (previous section).

We focus more on allowing users to build trees of project versions

## crdts
Tempo previously used Automerge as its primary storage mechanism for application data.

Automerge was useful for two things:
- merging together concurrent insertions into a key-value map together
  - e.g. user A adds kv pair I1 to map M, user B adds kv pair I2 to M, M = { I1, I2 }
- deterministic tie breaking of values concurrently written to

I think both benefits were a very minimal part of what Automerge offered, and can be implemented internally in Tempo.
I'd also like Tempo to have a notification system, and it's been tricky to figure out how I could implement this with Automerge.
Essentially the issue seems to be taking diffed changes and converting them into notifications. For example with Automerge i'd have to do:

`PatchLog` (set of diffs from prev state) -> categorization system for individual changes (what changed?) -> notification system

In Automerge, If I modelled comments as being a map inside of some kind of `Note` object, I would need to recognize that an insertion in this map is a creation of a comment. This feels like one too many moving parts.

I hope to make this simpler in Tempo and have something like:

set of newly added Tempo session objects -> notification system

Overall, I don't think Automerge is a good fit for Tempo, but it should be strongly considered if Tempo needs to store realtime editable text or if Tempo itself were to become a DAW :).

Tempo is shifting towards being a Git-like simplistic CRDT.