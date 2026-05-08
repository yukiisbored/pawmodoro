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
            work_duration: 5,
            short_break_duration: 2,
            long_break_duration: 3,
            rounds_before_long_break: 4,
        }
    }
}
