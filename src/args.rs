// Copyright ⓒ 2019-2021 Lewis Belcher
// Licensed under the MIT license (see LICENSE or <http://opensource.org/licenses/MIT>).
// All files in the project carrying such notice may not be copied, modified, or
// distributed except according to those terms

use clap::{crate_version, App, Arg};
use std::path::{Path, PathBuf};

pub struct Args {
	pub config: Option<PathBuf>,
}

pub fn collect() -> Args {
	let matches = App::new("Rust Blocks")
		.version(crate_version!())
		.author("Lewis B. <gitlab.io/lewisbelcher>")
		.about("A simple i3blocks replacement written in Rust.")
		.arg(
			Arg::with_name("config")
				.short("c")
				.long("config")
				.help("Config file to use.")
				.takes_value(true),
		)
		.get_matches();

	Args {
		config: matches
			.value_of("config")
			.map_or_else(default_config, |x| Some(Path::new(x).to_path_buf())),
	}
}

/// Get the default config to use.
fn default_config() -> Option<PathBuf> {
	for path in &[
		dirs::config_dir().unwrap().join("rs-blocks/config"),
		dirs::home_dir().unwrap().join(".rs-blocks"),
	] {
		if path.is_file() {
			return Some(path.to_path_buf());
		}
	}
	None
}
