use std::process;
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
			.map_or_else(default_config, check_config)
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

/// Check the given config is an existing file
fn check_config(path: &str) -> Option<PathBuf> {
	let path = Path::new(path);
	if !path.is_file() {
		eprintln!("No such file: {}", path.display());
		process::exit(1);
	}
	Some(path.to_path_buf())
}
