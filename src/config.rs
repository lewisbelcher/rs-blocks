use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
	#[serde(default = "default_background")]
	time_fmt: String,
}

fn default_time_fmt() -> String {
	"%a %d %b <b>%H:%M:%S</b>".to_string()
}
