//! Config resolution: merge layered `Tree` + overrides into a `Resolved` view
//! with per-field provenance. Pure logic; no I/O, no binding lookups.
//!
//! See `spec/decisions/007-config-resolution-redesign.md`.

mod merge;
mod resolved;
mod skills;
mod source;

pub use merge::merge;
pub use resolved::Resolved;
pub use skills::{Collision, Decision, Entry, UnknownPattern, resolve_skills};
pub use source::{Source, Sourced};
