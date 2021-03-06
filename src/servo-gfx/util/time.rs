// Timing functions.
use std::time::precise_time_ns;

pub fn time<T>(msg: &str, callback: fn() -> T) -> T{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = ((end_time - start_time) / 1000000u64) as uint;
    if ms >= 5 {
        debug!("%s took %u ms", msg, ms);
    }
    return move val;
}


