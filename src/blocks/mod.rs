// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

// derive proc_macro from exernal crate:
pub use rs_blocks_derive::Configure;

pub mod battery;
pub mod block;
pub mod brightness;
pub mod cpu;
pub mod memory;
pub mod network;
pub mod time;
pub mod volume;

pub use block::{Block, Configure, Message, Sender};
