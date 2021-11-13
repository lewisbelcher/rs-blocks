// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

//! # Time block
//!
//! Use this block to get time monitoring in the status bar.
//!
//! Typical configuration:
//!
//! ```toml
//! [time]
//! ```
//!
//! ## Configuration options
//!
//! - `name`: Name of the block (must be unique)
//! - `period`: Default update period in seconds (extra updates may occur on
//!    event changes etc)
//! - `format`: Strftime format string for specifying the time format

use crate::blocks::{Block, Configure, Message, Sender};
use chrono::prelude::*;
use serde::Deserialize;
use std::thread;
use std::time::Duration;

#[derive(Configure, Deserialize)]
pub struct Time {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	#[serde(default = "default_format")]
	format: String,
}

fn default_name() -> String {
	"time".to_string()
}

fn default_period() -> f32 {
	1.0
}

fn default_format() -> String {
	"%a %d %b <b>%H:%M:%S</b>".to_string()
}

impl Sender for Time {
	fn add_sender(&self, channel: crossbeam_channel::Sender<Message>) -> anyhow::Result<()> {
		let name = self.get_name();
		let format = self.format.clone();
		let period = self.period;
		let mut block = Block::new(name.clone(), true);

		thread::spawn(move || loop {
			block.full_text = Some(Local::now().format(&format).to_string());
			channel.send((name.clone(), block.to_string())).unwrap();
			thread::sleep(Duration::from_secs_f32(period));
		});

		Ok(())
	}
}
