# Design

This document covers the design of Tempo's file sync provider.

## Overview
In Tempo, we use file sync services to sync a session, and also use it as a mechanism for **message passing**.

## Assumptions
From what I know, file sync services are not intended to be used as transport layers for applications. Very few applications explicitly use file sync services as supported transport layers. Despite this, I believe the guarantees that file sync services provide are enough to make them usable as transport layers.

https://tonsky.me/blog/crdt-filesync/

## Sync Service Considerations
Main considerations:

1. Avoid conflicting writes
2. Hard-code certain filenames in situations where there might be write conflicts

### Conflicting Writes
To handle write conflicts, many sync services will use last-writer-wins to determine the state of the file.

Along with this, many sync services will save a copy of the file with the losing write and will add a prefix/postfix signifying it's a conflicted version. For example, if two users edited `file.txt` at the same time, the last write would become `file.txt` while the other might be `file-conflict.txt`.

### Missing Local Files
Cloud-based sync services often remove local copies of files and will only keep cloud copies. Usually these services are smart about downloading files on-demand when they're `open(2)`ed. If a user tries to open a file when offline, it obviously cannot be downloaded from the cloud. I figure most sync services provide means to always download local copies of files, so users need to make sure they enable these settings.

## Folder Structure
In the fs provider, a folder corresponds to a session.

Shared folder layout is as follows:

- `[folder].session`: user-created directory, name can be anything but ends with .session
  - `[uuid]`: messages sent by a user with this uuid
    - `[hex counter]`: a message from this user

Shared folders are essentially used as a messaging mechanism. Each user passes messages in their respective folder.

A session-wide Lamport clock is used in the session for ordering messages. In the case of concurrent messages, a unix timestamp is used as a tiebreaker for ordering events. If unix timestamps are equal, we just compare uuids.
