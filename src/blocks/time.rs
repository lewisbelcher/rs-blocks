use chrono::prelude::*;

use std::thread;
use std::time::Duration;

use crate::blocks::Block;

const FMT: &str = "%a %d %b <b>%H:%M:%S</b>";

pub fn add_sender(
	name: &'static str,
	s: crossbeam_channel::Sender<(&'static str, String)>,
) -> &'static str {
	let mut block = Block::new(name, true);

	thread::spawn(move || loop {
		block.full_text = Some(Local::now().format(FMT).to_string());
		s.send((name, block.to_string())).unwrap();
		thread::sleep(Duration::from_secs(1));
	});
	name
}
