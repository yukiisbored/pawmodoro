#[derive(Debug, Clone)]
pub struct Config {
    pub work_duration: u64,
    pub short_break_duration: u64,
    pub long_break_duration: u64,
    pub rounds_before_long_break: u64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            work_duration: 25 * 60,
            short_break_duration: 5 * 60,
            long_break_duration: 15 * 60,
            rounds_before_long_break: 4,
        }
    }
}
