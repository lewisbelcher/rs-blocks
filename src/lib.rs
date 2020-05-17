pub mod ema;
pub mod file;
pub mod blocks;

pub fn str_to_f32(string: &str) -> f32 {
	string.trim().parse().unwrap()
}
