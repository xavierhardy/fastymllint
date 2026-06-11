// fastymllint — a fast, drop-in replacement for yamllint written in Rust.
// Based on the work of yamllint (Copyright (C) 2016 Adrien Vergé and
// contributors, GPL-3.0-or-later); the scanner and parser are derived from
// PyYAML (MIT).
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![forbid(unsafe_code)]

//! fastymllint — a fast, drop-in replacement for yamllint written in Rust.
//!
//! Diagnostics — messages, positions, severities and ordering — are
//! byte-for-byte compatible with yamllint. On top of that, fastymllint adds
//! JSON and plain-text output formats, parallel file processing, and
//! auto-fix / auto-format modes with dry-run support.

pub mod config;
pub mod decoder;
pub mod fix;
pub mod linter;
pub mod output;
pub mod pyyaml;
pub mod rules;
pub mod runner;

pub use config::{YamlLintConfig, YamlLintConfigError};
pub use linter::{Level, LintProblem};
