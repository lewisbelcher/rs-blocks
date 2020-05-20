use crate::blocks::Block;
use crate::utils;
use std::thread;

pub fn add_sender(
	name: &'static str,
	s: crossbeam_channel::Sender<(&'static str, String)>,
) -> &'static str {
	let mut block = Block::new(name, true);
	let monitor = utils::monitor_command("brightnessctl", &["g"], 10000);

	thread::spawn(move || {
		for output in monitor {
			block.full_text = Some(if let Ok(num) = utils::str_to_f32(&output) {
				format!(" {:.0}%", num / 75.0)
			} else {
				output
			});
			s.send((name, block.to_string())).unwrap();
		}
	});
	name
}
