use crate::blocks::Block;
use crate::utils;
use std::thread;

pub fn add_sender(
	name: &'static str,
	s: crossbeam_channel::Sender<(&'static str, String)>,
) -> &'static str {
	let mut block = Block::new(name, true);
	let mut monitor = utils::monitor_command("brightnessctl", &["g"], 10.0);
	let recv = utils::wait_for_signal(signal_hook::SIGUSR1, 10.0);

	thread::spawn(move || loop {
		let output = monitor.read();
		block.full_text = Some(if let Ok(num) = utils::str_to_f32(&output) {
			format!(" {:.0}%", num / 75.0)
		} else {
			output
		});
		s.send((name, block.to_string())).unwrap();
		recv.recv().unwrap();
	});
	name
}
