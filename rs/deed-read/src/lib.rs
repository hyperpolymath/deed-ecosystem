// SPDX-License-Identifier: MPL-2.0
//! Reader for the estate's `.deed` format.
//!
//! * [`syntax`] parses any deed against the normative grammar
//!   (`1-formats/deed/spec/abnf/deed.abnf` in `hyperpolymath/standards`).
//! * [`updates`] reads the `(updates …)` repo-deed vocabulary
//!   (`1-formats/deed/vocabulary/updates.adoc`), failing closed.

pub mod syntax;
pub mod updates;

pub use syntax::{parse, Node, Value};
