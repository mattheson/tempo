# Design

This document covers the design of Tempo's file sync provider.

Notes in this document might be useful for developers who are interested in developing their own applications which use file sync services as a mechanism for synchronizing application data between their clients.

## Overview
In Tempo, we solely use file sync services as a mechanism for **message passing**. Messages only contain objects.

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

- `[folder]`: user-created directory, name can be anything
  - `tempo-fs-session`: tempo directory, holds all data
    - `info`: immutable metadata about this folder, json, created on session creation
    - `[uuid]`: messages sent by a user with this uuid
      - `[hex counter]`: a message from this user

Shared folders are essentially used as a messaging mechanism. Each user passes messages in their respective folder.

A session-wide Lamport clock is used in the session for ordering messages. In the case of concurrent messages, a unix timestamp is used as a tiebreaker for ordering events. If unix timestamps are equal, we just compare uuids.

## diffing folder state
figuring out how to efficiently diff shared folder state efficiently, avoiding lots of memory/time usage.
this functionality is needed in order to implement notifications.

**short summary: each user has their own folder and messages are just files named with a counter that increments**

say we have folder F shared between users U_a and U_b.
U_a and U_b have their own subdirectories in F. to share data between both users, our application creates files in these subdirectories.
F is shared between Ua and Ub using a third party sync service.

each user can update their copy of F. in this problem, "update" just means creating a file.
our sync service guarantees that after Z updates to their copy of F by user U_x, all other users ( {users} - U_x ) will receive the updated state of F
(there is an upper bound to the number of updates each user can make to F before other users see changes)

this sync service does not guarantee that updates to the state of F are received in order by other clients.

we have a small set of operations we can perform on F's filesystem:
(n is number of files in subdir)
- num_files("F/subdir")
  - O(1) (assuming this is O(1) in filesystems)
- file_exists("F/subdir/file")
  - O(log n) (btree search is log n)
- get_files("F/subdir") -> set of filenames
  - O(n) space/time

problem: user Ua creates a file in F/Ua. how can we design our system so Ub can efficiently identify the new file after the state of F is synchronized?

#### O(n log n) approach
this solution properly handles extra, random files in subdirectories

each user names their messages with an incrementing int

F: our shared folder

M: set of missing files named with int that haven't synced

N: largest int-named file that we've successfully seen in Ua's subdir

S: size of Ua subdir (last known number of files in directory)

```
# returns set of new files that need to be indexed
function tempo_reindex(F, M, N, S):
  n = num_files("F/Ua") - S

  if n == 0: return [] 

  new_files = []
  extra_files = 0

  for f in F/Ua:
    if int_named(f):
      if f not in M and f < N: continue # we've indexed this at some point

      else if f in M:
        new_files += f
        M -= f

      else if f > N:
        new_files += f
        if f > N + 1:
          M += enumerate(N + 1 to f - 1)
        N = f
      
    else:
      extra_files += 1

  return new_files

```
in applications where users might be creating millions of files this approach would not be appropriate, something closer to the approach below would be better

sync services or users tampering with the subdirectories might create extra files or remove extra files (for example opening a directory in Finder and creating .DS_Store)

#### bad O(log n)
*something like this solution might be more practical for applications where a developer might create A LOT of files and needs to avoid a full reindex of the folder contents. i dont anticipate needing this approach for tempo.* 

like previous solution, Ua and Ub each name their files using an integer that increments once after they write a file

say the current number of files in F/Ua known by Ub is Nua
say that the set of files missing for Ub is known as M

to initially identify the change, Ub runs num_files("F/Ua") and sees that:

num_files("F/Ua") = Nua + 1

there are two possibilities for what this new file might be named
- \in M
- Nua + 1, ..., Nua + Z

|M| <= Z - 1

(i think)

worst case is that Ub performs 2Z - 1 file_exists() lookups which is technically O(log n)

Z bound is not necessarily a safe assumption with sync services, it could be possible to create a safe Z guarantee on the application level somehow? like not allowing user to create more messages if other user hasn't receieved any