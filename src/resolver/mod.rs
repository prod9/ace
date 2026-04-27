//! Config resolution: merge layered `Tree` + overrides into a `Resolved` view
//! with per-field provenance. Pure logic; no I/O, no binding lookups.
//!
//! See `spec/decisions/007-config-resolution-redesign.md`.

// Until the binding switchover lands (C2), nothing outside this module consumes
// these types; the unused symbols are intentional scaffolding.
#![allow(dead_code, unused_imports)]

mod merge;
mod resolved;
mod skills;
mod source;

pub use merge::merge;
pub use resolved::Resolved;
pub use skills::{
    Collision, Decision, Entry, Field, Op, Resolution, ResolvedSkill, UnknownPattern, resolve_skills,
};
pub use source::{Source, Sourced};
