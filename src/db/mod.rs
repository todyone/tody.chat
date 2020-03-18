//! This module contains functions for basic interactions
//! with a database.
//! Contol of data consistency provided by a `database` actor.

mod v0001;
pub use v0001::{Dba, Session, User};
