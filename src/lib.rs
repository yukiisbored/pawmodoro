pub struct PomodoroConfig {
    pub work_duration: u64,
    pub short_break_duration: u64,
    pub long_break_duration: u64,
    pub cycles_before_long_break: u32,
}

pub struct TimerState {
    pub is_running: bool,
    pub time_remaining: u64,
}

pub enum PomodoroState {
    Work(TimerState),
    ShortBreak(TimerState),
    LongBreak(TimerState),
}

pub struct PomodoroTimer {
    pub config: PomodoroConfig,
    pub state: PomodoroState,
    pub completed_cycles: u32,
}

pub mod timer;