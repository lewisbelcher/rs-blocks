// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

//! # Battery block
//!
//! Use this block to get battery monitoring in the status bar.
//!
//! Typical configuration:
//!
//! ```toml
//! [battery]
//! ```
//!
//! ## Configuration options
//!
//! - `name`: Name of the block (must be unique)
//! - `period`: Default update period in seconds (extra updates may occur on
//!    event changes etc)
//! - `alpha`: Weight for the exponential moving average of value updates
//! - `path_to_charge_now`: Path to file containing current charge (usually
//!    something like `/sys/class/power_supply/BAT0/charge_now`)
//! - `path_to_charge_full`: Path to file containing charge value when full
//!    (usually something like `/sys/class/power_supply/BAT0/charge_full`)
//! - `path_to_status`: Path to file containing battery status (usually
//!    something like `/sys/class/power_supply/BAT0/status`)

use crate::blocks::{Block, Configure, Message as BlockMessage, Sender, ValidatedPath};
use crate::{ema, utils};
use anyhow::Context;
use serde::Deserialize;
use std::fs;
use std::thread;
use std::time::Instant;

#[derive(Configure, Deserialize)]
pub struct Battery {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	#[serde(default = "default_alpha")]
	alpha: f32,
	#[serde(default = "default_path_to_charge_now")]
	path_to_charge_now: ValidatedPath,
	#[serde(default = "default_path_to_charge_full")]
	path_to_charge_full: ValidatedPath,
	#[serde(default = "default_path_to_status")]
	path_to_status: ValidatedPath,
}

fn default_name() -> String {
	"battery".to_string()
}

fn default_period() -> f32 {
	0.6
}

fn default_alpha() -> f32 {
	0.8
}

fn default_path_to_charge_now() -> ValidatedPath {
	ValidatedPath("/sys/class/power_supply/BAT0/charge_now".to_string())
}

fn default_path_to_charge_full() -> ValidatedPath {
	ValidatedPath("/sys/class/power_supply/BAT0/charge_full".to_string())
}

fn default_path_to_status() -> ValidatedPath {
	ValidatedPath("/sys/class/power_supply/BAT0/status".to_string())
}

impl Sender for Battery {
	fn add_sender(&self, channel: crossbeam_channel::Sender<BlockMessage>) -> anyhow::Result<()> {
		let name = self.get_name();
		let max = get_max_capacity(&self.path_to_charge_full.0)?;
		let (tx, rx) = crossbeam_channel::unbounded();
		let mut sremain = "...".to_string();
		let mut last_status_change = 0;
		let mut remaining = ema::Ema::new(self.alpha);
		let (mut current_charge, mut current_status) = initialise(
			&self.path_to_charge_now.0,
			&self.path_to_status.0,
			self.period,
			tx,
		)?;
		let mut then = Instant::now();
		let mut fraction = (current_charge / max).min(1.0);
		let mut block = Block::new(name.clone(), true);
		let symbol = get_symbol(current_status, fraction);

		if current_status == Status::Full {
			block.full_text = Some(create_full_text(symbol, fraction, "Full"));
			channel.send((name.clone(), block.to_string())).unwrap();
		}

		thread::spawn(move || loop {
			let message = rx.recv().unwrap();
			log::debug!("{:?}", message);
			let now = Instant::now();

			match message {
				Message::Charge(charge) => {
					sremain = if last_status_change == 0 {
						remaining.reset();
						"...".to_string()
					} else {
						let elapsed = now.duration_since(then).as_secs() as f32 / 60.0;
						let gap = match current_status {
							Status::Charging => max - charge,
							Status::Discharging => charge,
							Status::Full => 0.0,
							Status::NotCharging => charge,
							Status::Unknown => charge,
						};
						if charge == current_charge {
							continue;
						}
						let rate = (charge - current_charge).abs() / elapsed;
						log::info!("rate = {}", rate);
						minutes_to_string(remaining.push(gap / rate))
					};

					then = now;
					current_charge = charge;
					last_status_change += 1;
					fraction = (current_charge / max).min(1.0);
				}
				Message::Status(status) => {
					if status != current_status {
						last_status_change = 0;
						current_status = status;
						sremain = "...".to_string();
					}
				}
			}
			if current_status == Status::Full {
				sremain = "Full".to_string();
			}
			let symbol = get_symbol(current_status, fraction);
			block.full_text = Some(create_full_text(symbol, fraction, &sremain));
			channel.send((name.clone(), block.to_string())).unwrap();
		});

		Ok(())
	}
}

fn get_max_capacity(path: &str) -> anyhow::Result<f32> {
	let contents = fs::read_to_string(&path).context(format!("Could not read path '{}'", path))?;
	utils::str_to_f32(&contents).context(format!("Could not parse contents of '{}'", path))
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
	Charging,
	Discharging,
	Full,
	NotCharging,
	Unknown,
}

#[derive(Debug, Clone, Copy)]
enum Message {
	Charge(f32),
	Status(Status),
}

/// Convert a string to a status.
fn str_to_status(s: &str) -> anyhow::Result<Message> {
	let s = s.trim();
	match s {
		"Charging" => Ok(Message::Status(Status::Charging)),
		"Discharging" => Ok(Message::Status(Status::Discharging)),
		"Full" => Ok(Message::Status(Status::Full)),
		"Not charging" => Ok(Message::Status(Status::NotCharging)),
		"Unknown" => Ok(Message::Status(Status::Unknown)),
		_ => anyhow::bail!("Unknown status {}", s),
	}
}

/// Convert a string to a charge enum.
fn str_to_charge(s: &str) -> anyhow::Result<Message> {
	let value = s
		.trim()
		.parse()
		.context(format!("Unexpected value for charge '{}'", s))?;
	Ok(Message::Charge(value))
}

/// Continuously monitor `f` for changes, when a change occurs or more than 10
/// checks have occurred, pipe its contents through `parse_fn` and send the
/// results over the sender `tx`.
fn looper<F, T>(tx: crossbeam_channel::Sender<Message>, mut f: utils::Monitor<T>, parse_fn: F)
where
	F: 'static + Fn(&str) -> anyhow::Result<Message> + Send,
	T: 'static + FnMut() -> String + Send,
{
	thread::spawn(move || {
		let mut prev = f.read();
		let mut i = 0;

		for contents in f {
			log::debug!("Contents: {}", contents);
			if contents != prev || i > 10 {
				let parsed = parse_fn(&contents).expect(&format!(
					"Encountered bad value in battery file: '{}'",
					contents
				));
				tx.send(parsed).unwrap();
				prev = contents;
				i = 0;
			}
			i += 1;
		}
	});
}

/// Given a percentage of charge, wrap the string `s` in an appropriate colour.
fn wrap_in_colour(s: &str, fraction: f32) -> String {
	let colour = if fraction > 0.5 {
		format!("{:0>2x}ff00", 255 - (510.0 * (fraction - 0.5)) as i32)
	} else {
		format!("ff{:0>2x}00", (510.0 * fraction) as i32)
	};
	format!("<span foreground='#{}'>{}</span>", colour, s)
}

/// Given a percentage of charge, return an appropriate battery symbol.
fn get_discharge_symbol(fraction: f32) -> &'static str {
	if fraction > 0.90 {
		" "
	} else if fraction > 0.60 {
		" "
	} else if fraction > 0.40 {
		" "
	} else if fraction > 0.10 {
		" "
	} else {
		" "
	}
}

fn get_symbol(status: Status, fraction: f32) -> String {
	let s = match status {
		Status::Discharging => get_discharge_symbol(fraction),
		_ => " ",
	};
	wrap_in_colour(s, fraction)
}

/// Convert a float of minutes into a string of hours and minutes.
fn minutes_to_string(total: f32) -> String {
	let (mut hrs, mut mins) = (total / 60.0, total % 60.0);
	if mins >= 59.5 {
		hrs += 1.0;
		mins = 0.0;
	} else {
		mins = mins.round();
	}
	format!("{:.0}h{:02.0}m", hrs.floor(), mins)
}

/// Start watching the appropriate files for changes and return their current
/// contents.
fn initialise(
	path_to_charge_now: &str,
	path_to_status: &str,
	period: f32,
	tx: crossbeam_channel::Sender<Message>,
) -> anyhow::Result<(f32, Status)> {
	let mut charge_file = utils::monitor_file(path_to_charge_now.to_string(), period);
	let mut status_file = utils::monitor_file(path_to_status.to_string(), period);

	let current_charge = match str_to_charge(&charge_file.read())? {
		Message::Charge(value) => value,
		_ => unreachable!(),
	};

	let current_status = match str_to_status(&status_file.read())? {
		Message::Status(value) => value,
		_ => unreachable!(),
	};

	looper(tx.clone(), charge_file, str_to_charge);
	looper(tx.clone(), status_file, str_to_status);

	Ok((current_charge, current_status))
}

fn create_full_text(symbol: String, fraction: f32, remaining: &str) -> String {
	format!("{} {:.0}% ({})", symbol, fraction * 100.0, remaining)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn minutes_to_string_works() {
		env_logger::init();
		assert_eq!(minutes_to_string(302.2), "5h02m");
		assert_eq!(minutes_to_string(302.7), "5h03m");
		assert_eq!(minutes_to_string(60.0), "1h00m");
		assert_eq!(minutes_to_string(59.99), "1h00m");
		assert_eq!(minutes_to_string(60.5), "1h01m");
		assert_eq!(minutes_to_string(60.4999), "1h00m");
		assert_eq!(minutes_to_string(39.5), "0h40m");
	}

	#[test]
	fn test_wrap_in_colour() {
		let result = wrap_in_colour("a", 1.0);
		assert_eq!(result, "<span foreground=\'#00ff00\'>a</span>");

		let result = wrap_in_colour("a", 0.01);
		assert_eq!(result, "<span foreground=\'#ff0500\'>a</span>");
	}
}
