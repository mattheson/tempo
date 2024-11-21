# Design

## Ideas
### Collaboration and Version Management
Tempo aims to refine collaboration and version management for music producers.

Currently, you have to manually manage versions of your project files.

My typical process has been to create a quick copy of a project if I'm satisfied with my progress before making additional changes. Other producers like to use more elaborate file naming/folder hierarchy schemes.

This system works well when you're working on a project by yourself, but when you add collaboration into the mix, maintaining a file-naming based version management can get tricky.

Music producers also deserve something better than file management for version management/collaboration with others! Filenames and directories can be flat and boring, and it can be difficult to see the relationships between project files.

Versioning consists of two actions:
1. creating a new thing
2. adding a new version of the thing
  
These actions are similar to those of messaging systems:
1. sending a message
2. replying to a message

Tempo uses the concept of a "note". Like a message, a note can have an attachment. Currently, you can only attach a project file or audio file to a note.

To represent a new version of a project, a producer can create a note and reply to a note containing the previous version of the project. Replies can be whatever you want them to be as well! It doesn't have to be limited to version management.

Tempo provides the ability to reply to multiple notes to allow producers to "merge" projects or files together.

Producers often use messaging services to send project files back and forth, so this should be a familiar interface.
Along with the message-based interface, users have the ability to create channels to organize their different projects however they please.

This interface provides flexibility and very little friction for producers. Music production can be chaotic; you can be working on a new version of a song and it might turn into a new song altogether. The solution to this is simple: you send the project as a new message, maybe creating a new channel as well.

### Local-First
Tempo uses a local-first design to ensure it's long-lasting and doesn't depend on me running a web service.
Users can share data between each other using a file sync service.

### File and Plugin Synchronization
One issue that's emerged in my experiences collaborating with others are missing file references or plugin mismatches.
I dug into the internals of Ableton's project files to see what was possible to solve this issue. Ableton's project file format is simply gzipped XML, so it was very easy to figure out what was going on.

#### File References
Tempo automates Ableton's "Collect All and Save" feature by automatically copying referenced files in projects into Tempo's shared folders. Tempo also adjusts the file references inside of project files to point into a "Files" directory. When you create a copy of a project, Tempo copies all referenced files from the folder into the "Files" directory.

#### Plugin Synchronization
Tempo allows users to check whether they've used plugins in projects which their collaborators also have installed. To achieve this, Tempo reads Ableton's plugin database (and scans Audio Units on macOS) into a SQLite database. This database is copied into shared folders. Other users read from this database when adding a project to check whether the project contains any plugins that others are missing.

I still feel like Tempo doesn't completely address the problem of plugin synchronization. It would be preferable to *prevent* users from adding incompatible plugins into projects altogether. I see two possible means to do this in the future:

**1. Modifying Ableton's plugin database:** I don't like this solution, and it also seems to require a restart of Ableton for the changes to register.

**2. Monitoring changes, alerting users:** Tempo could monitor a project file. When the file is changed, it could be scanned to see if any plugins were added. If Tempo finds a plugin that other users are missing, it could show an alert to the user.

I'm going to look into implementing #2 in the future. For now, users can attach a project to a note and rescan it while they're editing it to check plugin references.

## Architecture
Since this application uses Tauri, Tempo consists of a WebView frontend and a Rust backend.
Generally the backend should do most data processing/data-intensive stuff. The frontend should just show stuff. Pretty standard.

## Data Model
Data in folders is mostly stored in Automerge documents, except for user-specific metadata.

All state is synchronized through third party sync services. Users will have to separately create folders and set up these sync services, then they add the shared folder to Tempo. Tempo needs to be designed to work well with these services. 
Tempo could also work if two users were to save the shared folder on a flash drive and pass it between each other. Users could also work on their own copies of folders and merge them together with their file explorer's merge functionality. Lots of things might be possible.

## Sync Services
Tempo expects users to use third-party sync services to synchronize shared folders between each other. This creates some problems.

One big problem is conflicting writes (two users writing to file at same time before syncing). Many sync services will use last-writer-wins to determine the state of the file.

Along with this, many sync services will save a copy of the file with the losing write and will add a prefix/postfix signifying it's a conflicted version. For example, if two users edited `file.txt` at the same time, the last write would become `file.txt` while the other might be `file-conflict.txt`.

Considering this, there are two important points to keep in mind:
1. Try to avoid conflicting writes
2. Hard-code certain filenames in situations where there might be write conflicts

Furthermore, sync services often remove local copies of files. Usually these services are smart about downloading files on-demand when they're `open(2)`ed. If a user tries to open a file when offline, it obviously cannot be downloaded from the cloud. I figure most sync services provide means to always download local copies of files, so users need to make sure they enable these settings.

## Folder Layout
The layout of a **shared Tempo folder** is as follows:

- `[folder]`: user-created directory, name can be anything
  - `tempo`: tempo directory, holds all data
    - `[uuid]`: data scoped to a user with this install ulid
      - `[ulid]`: this user's latest copy of the `[ulid]` automerge document
      - `session`: this user's latest copy of the session metadata
      - `data`: latest copy of this user's metadata
    - `files`: files referenced in this folder
      - `[sha256]`: a file

### 


## Data Directory
The layout of the per-client data directory is as follows:
- `[data dir]`: name of data directory, could vary depending on OS
  - `settings.json`: tauri kv store accessed only by frontend, stores frontend settings
  - `tempo.sqlite`: Tempo's internal sqlite database

## State Management
State management can be tricky with Tauri. You have to pass data between the frontend and backend using Tauri's command and event system.

*How do we read data on the frontend?*

The simplest solution I've found is to write **everything** into a SQLite database, and give the frontend readonly access to this database using Tauri's SQL plugin. This seems like the most flexible way for the frontend to read data. Directly using SQL on the frontend allows us to avoid writing a bunch of commands/events to pass data to the frontend.



## Further Considerations
- consider whether files should use headers instead of separate metadata files (`FileInfo`s/meta files)
  - eliminates a whole class of sync issues, sync services are good with atomically syncing files (users dont get access to partially-synced files afaik)
  - something simple like a JSON header at the top of a file might work
  - have to implement a custom tauri protocol for this probably to strip to the header for playing audio files
  - copying files might be slower
    - the header needs to be stripped
    - audio files can get very large and it's nice to not have to stream the whole file just to write it back to disk
    - might be tricks with sparse files or modern fs features to avoid streaming whole file?