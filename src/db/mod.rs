//! This module contains functions for basic interactions
//! with a database.
//! Contol of data consistency provided by a `database` actor.

pub mod types;

mod v0001;
pub use v0001::{Channel, Dba, DbaError, Session, User};
