File sync (fs) provider for Tempo.

This provider reads and writes to directories. A directory corresponds to a session in this provider.
Users can share using third-party file sync services.

## filesystem sessions
Filesystem sessions are extremely similar to Git's `.git` directory.

To safely store filesystem-based sessions in **sync services** and allow for **concurrent modification**:
- avoid concurrent writes as much as possible
  - use unique values in the kv store
  - have a guarantee that users create unique objects
    - e.g. storing uuid somewhere in blob should be sufficient
  - only modify files if you have a guarantee that other users won't modify them
- you need deterministic conflict resolution for any conflicting writes

### structure

A session corresponds to a directory. The name of this directory **must** end with ".session".

Structure overview:

- `[name].session`
  - `info`: info file
  - `store`: kv store
    - `[any subdirectories/any files]`
  - `objects`: objects directory
    - `[2 chars]`
      - `[remaining sha256 chars]`

### store directory
The `store` directory contains data which **should only be read/modified through the `Session` key value store interface**.

### the info file 
This is an immutable plaintext file that should be created upon the creation of the session.
Generally, this file **should not be modified**.

This file must follow this format:

```
[tempo session version number]\n
[name of application consuming/using this session][space][SemVer/some version number of this application/any string (not verified)]\n
[remaining contents can be anything]
```

This file should be a plaintext file.

### objects directory
The `objects` directory follows this structure. This is pretty much identical to Git's object store:

- `[first two chars of sha256]`
  - `remaining chars of sha256` <- an object
  - `remaining chars of sha256` <- an object
  - more objects...
- `[two char-named directory]`
  - more objects...
- more directories...

Objects must follow this format:

```
[optional object name]\0[object type string]\0[remaining bytes: zstd-compressed object]
```
