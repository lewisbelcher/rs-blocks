// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use crate::blocks::{Block, Configure, Message as Msg, Sender};
use crate::{ema, utils};
use serde::Deserialize;
use std::fs;
use std::thread;
use std::time::Instant;

const PATH: &str = "/sys/class/power_supply/BAT0";

#[derive(Configure, Deserialize)]
pub struct Battery {
	#[serde(default = "default_name")]
	name: String,
	#[serde(default = "default_period")]
	period: f32,
	#[serde(default = "default_alpha")]
	alpha: f32,
	#[serde(default = "default_charge_prefix")]
	charge_prefix: String,
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

fn default_charge_prefix() -> String {
	"charge".to_string()
}

impl Sender for Battery {
	fn add_sender(&self, channel: crossbeam_channel::Sender<Msg>) {
		let name = self.get_name();
		let max = get_max_capacity(&self.charge_prefix);
		let (tx, rx) = crossbeam_channel::unbounded();
		let mut sremain = "...".to_string();
		let mut last_status_change = 0;
		let mut remaining = ema::Ema::new(self.alpha);
		let (mut current_charge, mut current_status) =
			initialise(&self.charge_prefix, self.period, tx);
		let mut then = Instant::now();
		let mut fraction = (current_charge / max).min(1.0);
		let mut block = Block::new(name.clone(), true);
		let mut symbol = get_symbol(current_status, fraction);

		if current_status == Status::Full {
			block.full_text = Some(create_full_text(&symbol, fraction, "Full"));
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
			symbol = get_symbol(current_status, fraction);

			block.full_text = Some(create_full_text(&symbol, fraction, &sremain));
			channel.send((name.clone(), block.to_string())).unwrap();
		});
	}
}

fn get_max_capacity(charge_prefix: &str) -> f32 {
	let path = format!("{}/{}_full", PATH, charge_prefix);
	utils::str_to_f32(&fs::read_to_string(&path).unwrap()).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
	Charging,
	Discharging,
	Full,
	Unknown,
}

#[derive(Debug, Clone, Copy)]
enum Message {
	Charge(f32),
	Status(Status),
}

/// Convert a string to a status.
fn str_to_status(s: &str) -> Message {
	match s.trim() {
		"Charging" => Message::Status(Status::Charging),
		"Discharging" => Message::Status(Status::Discharging),
		"Full" => Message::Status(Status::Full),
		"Unknown" => Message::Status(Status::Unknown),
		_ => panic!("Unknown status {}", s),
	}
}

/// Convert a string to a charge enum.
fn str_to_charge(s: &str) -> Message {
	Message::Charge(s.trim().parse().unwrap())
}

/// Continuously monitor `f` for changes, when a change occurs or more than 10
/// checks have occurred, pipe its contents through `content_fn` and send the
/// results over the sender `tx`.
fn looper<F, T>(tx: crossbeam_channel::Sender<Message>, mut f: utils::Monitor<T>, content_fn: F)
where
	F: 'static + Fn(&str) -> Message + Send,
	T: 'static + FnMut() -> String + Send,
{
	thread::spawn(move || {
		let mut prev = f.read();
		let mut i = 0;

		for contents in f {
			if contents != prev || i > 10 {
				tx.send(content_fn(&contents)).unwrap();
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
fn minutes_to_string(remain: f32) -> String {
	let (hrs, mins) = (remain / 60.0, remain % 60.0);
	format!("{:.0}h{:02.0}m", hrs.floor(), mins)
}

/// Start watching the appropriate files for changes and return their current
/// contents.
fn initialise(
	charge_prefix: &str,
	period: f32,
	tx: crossbeam_channel::Sender<Message>,
) -> (f32, Status) {
	let mut charge_file = utils::monitor_file(format!("{}/{}_now", PATH, charge_prefix), period);
	let mut status_file = utils::monitor_file(format!("{}/status", PATH), period);

	let current_charge = match str_to_charge(&charge_file.read()) {
		Message::Charge(charge) => charge,
		_ => panic!("Unexpected contents of charge"),
	};

	let current_status = match str_to_status(&status_file.read()) {
		Message::Status(status) => status,
		_ => panic!("Unexpected contents of status"),
	};

	looper(tx.clone(), charge_file, str_to_charge);
	looper(tx.clone(), status_file, str_to_status);

	(current_charge, current_status)
}

fn create_full_text(symbol: &str, fraction: f32, remaining: &str) -> String {
	format!("{}{:.0}% ({})", symbol, fraction * 100.0, remaining)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn minutes_to_string_works() {
		assert_eq!(minutes_to_string(302.2), "5h02m");
		assert_eq!(minutes_to_string(302.7), "5h03m");
	}

	#[test]
	fn test_wrap_in_colour() {
		let result = wrap_in_colour("a", 1.0);
		assert_eq!(result, "<span foreground=\'#00ff00\'>a</span>");

		let result = wrap_in_colour("a", 0.01);
		assert_eq!(result, "<span foreground=\'#ff0500\'>a</span>");
	}
}
