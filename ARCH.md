# Architecture

This documents contains descriptions of basic architectural decisions.

## Roles of modules and data layout

* `Dba` - provides generic functions to control tables
* `EngineActor` - uses `Dba`, but keeps data consistency.
   For example every channel must have at least one member.
* `CtrlActor` and `LiveActor` - interacts with `EngineActor`, but handles user-specific
   tasks like finding Id of a user by name.