Definitions of Tempo's session/provider traits.

Sessions are basically the following:
- object store/content-addressable store
  - objects are always identified with sha256 hashes
- key-value store with following `Value` types:
  - `Map`: nested map
  - `Data`: bytes

Providers provide access to sessions, and must handle:
- updating Tempo's SQLite database
- peristance/retrieval of objects/values

This is basically the "plumbing" of Tempo.