//! # patchwork
//!
//! A library for diffing and patching sequences and nested structures.
//!
//! ## Features
//!
//! - **Myers diff** — efficient sequence diffing via the Myers algorithm
//! - **Recursive diff** — structural diffing of nested maps and sequences
//! - **Hunks** — group changes with context lines
//! - **Unified diff** — serialize and deserialize patches in unified diff format
//!
//! ## Quick Start
//!
//! A simple Vec of primitives can be diffed using Myers algorithm.
//! The diff can be transformed into a series of [`Hunk`]s.
//! Hunks can be transformed into a textual diff or applied to an input.
//!
//! `apply(&old, hunks(diff(&old, &new))) == Ok(new)`
//!
//! ```rust
//! use patchwork::myers::diff;
//! use patchwork::patch::{apply, hunks};
//! use patchwork::serialization::ToPatch;
//!
//! let old = vec!["hello", "world"];
//! let new = vec!["hello", "rust"];
//! let myers_edits = diff(&old, &new);
//!
//! let hunks = hunks(myers_edits);
//! let patch = hunks.to_patch(Some("lib.rs"), Some("lib.rs"));
//!
//! let equal_to_new = apply(&old, &hunks);
//! ```
//!
//! For nested structures a recursive diffing algorithm is provided.
//! The diff will return a list of [`Change`]s.
//! Changes can be transformed into Hunks and applied.
//! Changes cannot be serialized, since there is no consensus on a textual format.
//!
//! `apply(&old, hunks(diff(&old, &new))) == Ok(new)`
//!
//! ```rust
//! use std::collections::HashMap;
//! use patchwork::recursive::{apply, diff};
//! use patchwork::patch::hunks;
//!
//! let mut old = HashMap::new();
//! old.insert("Hello".to_string(), 1);
//! let mut new = HashMap::new();
//! new.insert("Hello".to_string(), 2);
//! let changes = diff(&old, &new);
//!
//! let equal_to_new = apply(&old, &changes);
//! ```

pub mod myers;
pub mod patch;
pub mod recursive;
pub mod serialization;
