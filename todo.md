# Todo
- Commands to change state
- command serializable
- make it generic
- rwlock for read/write acces
- must have a handler of C (generic over command)
- handler gets a mutable reference to state
- command que
- thread for writing commands to disk
- restore from disk
- snapshot restore
- create snapshot

## Questions
Should we create two states... let people read while others are writing and then stop giving access to old?
