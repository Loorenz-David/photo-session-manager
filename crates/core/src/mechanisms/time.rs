use crate::ports::clock::Clock;

pub fn now_ms(clock: &dyn Clock) -> i64 {
    clock.now_ms()
}
