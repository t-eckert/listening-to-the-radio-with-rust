use crate::decoder::ChuTime;

/// Print decoded CHU time to the terminal.
pub fn print_time(time: &ChuTime) {
    let year_str = match time.year {
        Some(y) => format!("{y}"),
        None => "????".to_string(),
    };

    println!(
        "  CHU Time: {}-{:03} {:02}:{:02}:{:02} UTC",
        year_str, time.day_of_year, time.hours, time.minutes, time.seconds
    );

    if let Some(dut1) = time.dut1 {
        println!("  DUT1: {dut1:+.1}s");
    }
}
