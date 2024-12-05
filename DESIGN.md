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

# Internals

## Similarities With Git
Tempo is very similar to Git. Tempo uses a content-addressable key-value store for storing files (files are stored named with their SHA256 hash).

Reused concepts from Git:
- objects
- refs

## Differences With Git
There have been previous attempts at creating collaboration/version management tools for producers (namely Blend.io and Splice Studio, I'm sure there's others). I'd like Tempo to be local-first and not rely on me running a service, hence my focus on ensuring users can use sync services to share sessions between each other. As a result of this, Tempo will be CRDTy in some parts of its design. Tempo used Automerge in its initial alpha design. Tempo should not be local-first-only, there should be some sort of Tempo-specific service users can run or use to share work with each other.

As a whole, it's best to reference Pijul or other distributed version management systems for design inspiration.

Tempo uses lots of specialized types of objects rather than a small set of objects.

Git is primarily focused on managing source code. Tempo is focused on managing audio files and project files. The workflow of music producers differs significantly from that of programmers. I have no concrete ideas for how

Considering this, I've tried to make Tempo's internals as simplistic as possible to allow for lots of extensibility in the future.


## notes
- tempo should move closer towards git
  - deadly simple folder structure
  - objects
- rust is plumbing, frontend is porcelain
- fs provider:
  - refs + objects folder, that's it