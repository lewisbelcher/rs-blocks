// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

//! # Rust Blocks
//!
//! A lightweight i3/sway status bar written in Rust.
//!
//! See the [README](https://crates.io/crates/rs-blocks) for project details, or
//! [blocks documentation](https://docs.rs/rs-blocks/latest/rs_blocks/blocks/index.html)
//! for available blocks and configurations.

#[macro_use]
extern crate lazy_static;

pub mod args;
pub mod blocks;
pub mod ema;
pub mod utils;
