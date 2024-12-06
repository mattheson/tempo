# tempo sessions

Tempo sessions are basically the following:
- object store/content-addressable store
  - objects are always identified with sha256 hashes
- key-value store with following `Value` types:
  - `Map`
  - `ObjRef`: reference to an object in the object store
  - `ValRef`: reference to another value in kv store
    - TODO: is this really needed? just leaving this in because refs can point to refs in Git
  - `Data`: arbitrary bytes
- references can be invalid/point to nothing
- some restrictions on key contents (see `id.rs`)

This crate provides a basic filesystem-based implementation of sessions. The `Session` trait can be used for creating other implementations.

**This is basically the "plumbing" of Tempo.**

Tempo sessions could be used to develop other collaborative applications. If you're interested in using this as a library let me know and I'll move it out of Tempo.

## filesystem sessions
This section covers implementation details of filesystem-based sessions.

Filesystem sessions are extremely similar to Git's `.git` directory.

To safely store filesystem-based sessions in **sync services** and allow for **concurrent modification**, developers should be mindful of the following:
- avoid concurrent writes as much as possible
  - use uuids in the kv store
  - create some guarantee that users create unique objects
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

As you can see, this is very similar to Git's `.git` directory.

### store directory
The data directory contains data which **should only be read/modified through the `Session` key value store interface**. There are no restrictions on the contents of this directory. It only needs to exist.

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
The `objects` directory must follow this structure. This is pretty much identical to Git's object store:

- `[first two chars of sha256]`
  - `remaining chars of sha256` <- an object
  - `remaining chars of sha256` <- an object
  - ...
- `[two char-named directory]`
  - ...
- ...

Objects must follow this format:

```
[optional object name]\0[object type string]\0[remaining bytes: zstd-compressed object]
```

The object file should be named with the sha256sum of the object's uncompressed bytes.

### `refs` directory
Ref files can be structured in any fashion within the `refs` directory (within any subdirectory structure), but importantly **all files** must follow this format:

The first line of a ref must be:

```
[sha256]
```

or

```
refs/[path to ref file]
```

or

```
data/[path to data file]
```

The following lines of the file can contain anything.

# TODOs
- delta objects
  - this crate aims to be a baseline/minimal implementation of sessions, object types will probably be implemented in a separate crate

# License
This crate is MIT OR Apache-2.0-licensed.