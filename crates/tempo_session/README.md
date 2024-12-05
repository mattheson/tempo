# Tempo sessions

Tempo sessions are basically just minimalistic object stores, similar to Git. You can think of a session as being analogous to a Git repository. Users store their data in sessions.

This crate provides plumbing for working with sessions, and a basic filesystem-based implementation of sessions.

A `Session` trait is provided, which can be used for other implementations.

**This is basically the "plumbing" of Tempo.**

Tempo sessions could be used to develop other collaborative applications. Other developers: let me know if you're interested in using this as a library and I'll move it out of Tempo.

## filesystem sessions
The provided filesystem-based implementation of sessions

### structure

A session corresponds to a directory. The name of this directory **must** end with ".session".

Structure overview:

- `[a directory].session`
  - `info`: info file
  - `data`: data directory
    - `[any subdirectories/any files]`
  - `objects`: objects directory
    - `[2 chars]`
      - `[remaining sha256 chars]`
  - `refs`: refs directory
    - `[any subdirectory structure]`
      - `[any filename]`

As you can see, this is very similar to Git's `.git` directory.

### data directory
The data directory contains arbitrary data. There are no restrictions on the contents of this directory.

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
Ref files can be structured in any fashion within the `refs` directory, but importantly **all files** themselves must follow this format:

The first line of a ref must be:

```
[sha256]\n
```

or

```
ref: refs/[path to ref file]\n
```

The rest of the file can contain anything.

# TODOs
- delta objects
  - this crate aims to be a baseline/minimal implementation of sessions, object types will probably be implemented in a separate crate

# License
This crate is MIT OR Apache-2.0-licensed.