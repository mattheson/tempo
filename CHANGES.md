# Release 0.0.0-alpha.1
This release contains some changes to Tempo's internals.

- Moved all metadata into a single automerge document
  - Moved username "allocation" into this metadata document
- Switched from using SQLite to share plugin data to JSON
  - Previous SQLite usage probably ventures close