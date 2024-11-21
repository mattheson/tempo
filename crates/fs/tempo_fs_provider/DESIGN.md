# Design

This document covers the design of Tempo's file sync provider.

## Sync Service Considerations
Main considerations:

1. Try to avoid conflicting writes
2. Hard-code certain filenames in situations where there might be write conflicts

### Conflicting Writes
To handle write conflicts, many sync services will use last-writer-wins to determine the state of the file.

Along with this, many sync services will save a copy of the file with the losing write and will add a prefix/postfix signifying it's a conflicted version. For example, if two users edited `file.txt` at the same time, the last write would become `file.txt` while the other might be `file-conflict.txt`.

### Missing Local Files
Cloud-based sync services often remove local copies of files and will only keep cloud copies. Usually these services are smart about downloading files on-demand when they're `open(2)`ed. If a user tries to open a file when offline, it obviously cannot be downloaded from the cloud. I figure most sync services provide means to always download local copies of files, so users need to make sure they enable these settings.

## Folder Structure
In the fs provider, a folder corresponds to a session.

Shared folder layout is as follows:

- `[folder]`: user-created directory, name can be anything
  - `tempo-session`: tempo directory, holds all data
    - `tempo`: immutable metadata about this folder, json
    - `[uuid]`: data scoped to a user with this install ulid
      - `[ulid]`: this user's latest copy of the `[ulid]` automerge document
      - `session`: this user's latest copy of the session metadata
      - `data`: latest copy of this user's metadata
    - `files`: files referenced in this folder
      - `[sha256]`: a file