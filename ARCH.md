# Architecture

This documents contains descriptions of basic architectural decisions.

## Roles of modules and data layout

* `Dba` - provides generic functions to control tables
* `EngineActor` - uses `Dba`, but keeps data consistency.
   For example every channel must have at least one member.
* `CtrlActor` and `LiveActor` - interacts with `EngineActor`, but handles user-specific
   tasks like finding Id of a user by name.

## Protocol

App interacts with a server using sequental (one request one respone) interaction protocol.
That's drastically simplifies debugging, support and size of codebase.
Also it allows to track and manage a queue of unsent messages.
Client tracks components that sent a request.

### Action, Reaction, Notification Model (ARNM)
Sequence: `Action` -> `Reaction`
In parallel: -> `Notification`
