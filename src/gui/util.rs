use bevy::prelude::*;

pub struct MsecTimer<const MSEC: usize>(Timer);

impl<const MSEC: usize> MsecTimer<MSEC> {
    pub fn tick(&mut self, time: &Time) -> bool {
        self.0.tick(time.delta()).just_finished()
    }
}

impl<const MSEC: usize> Default for MsecTimer<MSEC> {
    fn default() -> Self {
        let sec = MSEC as f32 / 1000.0;
        Self(Timer::from_seconds(sec, TimerMode::Repeating))
    }
}
