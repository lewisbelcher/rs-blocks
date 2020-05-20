use crate::blocks::Block;
use crate::utils;
use std::thread;

// TODO: Account for which device is being used

pub fn add_sender(
	name: &'static str,
	s: crossbeam_channel::Sender<(&'static str, String)>,
) -> &'static str {
	let mut block = Block::new(name, true);
	let re = regex::Regex::new(r"\[(?P<percent>\d+%)\] \[(?P<status>on|off)\]").unwrap();
	let monitor = utils::monitor_command("amixer", &[], 10000);

	thread::spawn(move || {
		for output in monitor {
			block.full_text = Some(if let Some(captures) = re.captures(&output) {
				if captures.name("status").unwrap().as_str() == "on" {
					format!(" {}", captures.name("percent").unwrap().as_str())
				} else {
					"".to_string()
				}
			} else {
				output
			});
			s.send((name, block.to_string())).unwrap();
		}
	});
	name
}
