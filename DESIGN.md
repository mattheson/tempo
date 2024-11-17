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
One limitation is merging. Very often, producers might take two projects and combine parts of them together. Right now, a note can only be a reply to one other note. It could be possible to allow a note to be a reply to multiple notes.
Producers often use messaging services to send project files back and forth, so this should be a familiar interface.
Along with the message-based interface, users have the ability to create channels to organize their different projects however they please.
This interface provides flexibility and very little friction for producers. Music production can be chaotic; you can be working on a new version of a song and it might turn into a new song altogether. The solution to this is simple: you send the project as a new message, maybe creating a new channel as well.

### Local-First
I want people to actually use Tempo. There are two barriers which could pose an issue with achieving this:

**1. Cost:**
Inevitably, if Tempo costed money, less people would use it. Tempo needs to be free.

**2. Web Services:**
It costs money to run a web service. I don't have infinite money to run a web service. Storing lots of project files somewhere in the cloud could easily get very expensive. Running a web service would also create security concerns; I don't want to get hacked and leak other producers' work.

After some research, I realized that using third-party file sync services (Dropbox, iCloud Drive, Syncthing, etc.) could be a reliable mechanism for syncing data between users. Many producers are familiar with using these services already as well.

A classmate of mine (thanks Matty!) suggested I looked into CRDTs as a means of storing synchronized data between users. This led me to Automerge, which is Tempo's primary mechanism for storing data which could be modified. As of right now, there isn't much mutability present in Tempo; once you send a note or create a channel, it's there forever. Automerge should hopefully enable mutability of this data in the future.

The core unit that's shared between users are **folders**. These folders can be synced using file sync services.

### Issues with Local-First
There are a few issues with the local-first file sync solution.

Firstly, file sync services can have a lot of latency, not allowing for realtime collaboration. There could be plenty of situations

Secondly, it can be desirable to have your work backed up to the cloud.

Lastly, the behavior of sync services (especially in the)


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
Data in folders is stored in Automerge documents, JSON and SQLite databases.

All state is synchronized through third party sync services. Users will have to separately create folders and set up these sync services, then they add the shared folder to Tempo. Tempo needs to be designed to work well with these services. 
Tempo could also work if two users were to save the shared folder on a flash drive and pass it between each other. Users could also work on their own copies of folders and merge them together with their file explorer's merge functionality. Lots of things might be possible.

### Storing Documents/Changes
Automerge documents are saved to disk using its optimized binary format.
Documents are always saved with their SHA256 hash as their name.
Documents are always stored in their own directory.

Clients can keep track of the latest known hash of Automerge documents. Whenever this hash changes, Tempo could recognize that the document has changed in some way. This could be used to implement notifications in the future.

Typically, the layout of a directory holding an automerge document will look as follows:
- directory: directory named with a ulid or `meta`
  - `[sha256 hash]`: the actual Automerge document

However, it's possible that the directory could contain multiple files if users independently make changes to the file at the same time. This would look something like this:
- directory
  - `[sha256 hash]`
  - `[sha256 hash 2]`

Automerge helps us address this synchronization issue. We simply use Automerge's merge functionality to merge the multiple documents together and save it as a new document. Once the new document is saved, we can delete the previous versions.

It's possible that this could result in some very obscure edge case synchronization issues. Generally, if the new document is saved before anything is deleted, this should work most of the time. It's probably worth testing this more.

All credit goes to Alex Good for telling me about this trick!

Previously, I stored Automerge documents as individual change files. This means that in order to read/write a document, the document needs to be built from scratch using these change files. I found that this could really slow stuff down when you had lots of change files. However, in Tempo, there should really be no situation where users are making excessive amounts of changes to a single document.

Lastly, each user needs their own actor id. In Automerge, an actor id is essentially a username which is attached to changes. Tempo's usernames act as actor ids. Actor ids/usernames are user-selected, within ASCII/length restraints. The presence of the `shared.sqlite` db is used as a very basic lock on a username/actor id. Clearly, there are situations where two users could be using the same username and are modifying the folder. Tempo is able to handle this. Each setup of Tempo will result in the generation of an installation-specific ulid. `shared.sqlite` will store a copy of the user's ulid. Tempo can check whether another user is using their desired username by comparing the ulid against any existing `shared.sqlite` database. If the ulids differ, another user is using the desired username/actor id.

I don't see username sync issues being a big problem with Tempo, but it would be cool to support having lots of producers use the same folder at once.

### Sync Services
Tempo expects users to use third-party sync services to synchronize shared folders between each other. This creates some problems.

One big problem is conflicting writes (two users writing to file at same time before syncing). Many sync services will use last-writer-wins to determine the state of the file.

Along with this, many sync services will save a copy of the file with the losing write and will add a prefix/postfix signifying it's a conflicted version. For example, if two users edited `file.txt` at the same time, the last write would become `file.txt` while the other might be `file-conflict.txt`.

Considering this, there are two important points to keep in mind:
1. Try to avoid conflicting writes
2. Hard-code certain filenames in situations where there might be write conflicts

Furthermore, sync services often remove local copies of files. Usually these services are smart about downloading files on-demand when they're `open(2)`ed. If a user tries to open a file when offline, it obviously cannot be downloaded from the cloud. I figure most sync services provide means to always download local copies of files, so users need to make sure they enable these settings.

### Documents
Each Automerge document is identified with a [ULID](https://github.com/ulid/spec). Since ULIDs have timestamps built in, it makes the process of sorting notes in chronological order very easy. Channels and notes are identified with ULIDs.

Tempo folders contain two kinds of Automerge documents:
- **channel:** document which contains metadata about a channel (the name of the channel, visibility)
- **note:** document which contains text and/or an attachment (project file or audio file)
  - comments can also be added to notes

This might seem very granular but it seems to be the most straightforward data model and I see no major problems with it. There could be problems I don't see yet. Previously I had stored all the state of a folder in one big Automerge document. Since Automerge documents must be fully loaded into memory in order to edit them, this uses an unacceptable amount of memory if a folder gets really big (when there's lots of files/messages/channels).

### Folder Layout
The layout of a **shared Tempo folder** is as follows:

- `[folder]`: user-created directory, name can be anything
  - `tempo`: tempo directory, holds all data
    - `info`: doc which stores metadata about this folder
    - `channels`: stores **channel directories**
      - `global`: stores global channel
        - has same layout as example channel folder below, but without metadata document
      - `[ulid]`: a **channel directory**
        - `aa`: a directory which holds **notes** where ulid hash part starts with 'aa'
          - `[ulid]`: a directory containing a note added in this channel
            - `[sha256]`: the Automerge note document
    - `files`: stores copies of files
      - `aa`: stores files starting with sha256 starting with `aa`
        - `[sha256]`: a directory corresponding to a file starting with `aa` in sha256 hash, named with the hash
          - `file`: the file itself
          - `meta`: JSON which holds file metadata. this metadata is immutable
    - `clients` : client-specific metadata
      - `[install ulid]`: metadata for a user with this install ulid, json

Importantly, all folders which store an automerge document will always be in the following format:

- directory
  - sha256 hash (automerge documents)
  - (possibly other sha256-named documents)

As previously stated, if there are multiple sha256 hash-named documents, they will be merged together and saved as one document.

The reason we split files between directories is to even the load of files. This is similar to Git's object store design. Some filesystems have limits on the number of files that can be held in a directory, and apparently this design can improve filesystem performance. This does result in a complicated folder structure, but it should hopefully be worth it. I assume this probably doesn't provide much of a benefit on modern filesystems, but it should hopefully make it possible for Tempo folders to be somewhat safely stored on FAT32 volumes or other folder filesystems.

### Data Directory
The layout of the per-client data directory is as follows:
- `[data dir]`: name of data directory, could vary depending on OS
  - `folders.json`: a listing of folders known by Tempo and user's install ulid
  - `tempo.json`: tauri kv store accessed only by frontend, stores frontend settings
  - `shared.sqlite`: the latest scan of plugins. copied into folders.

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