use std::time::Instant;

use calico::parser::{component::icalendar_stream, escaped::AsEscaped};

/// Takes a file path as an argument, parses it, and then prints some data about
/// how long the parsing took, and whether it parsed the entire file or not.
pub fn main() {
    let mut args = std::env::args();
    let _ = args.next();
    let path = args.next().unwrap();

    let load_start = Instant::now();
    let input = std::fs::read_to_string(path).unwrap();
    let mut input = input.as_escaped();

    let parse_start = Instant::now();
    let calendars = icalendar_stream::<_, winnow::error::InputError<_>>(&mut input)
        .unwrap();

    let end = Instant::now();
    eprintln!("parsed {} calendar object(s)", calendars.len());
    for (i, cal) in calendars.iter().enumerate() {
        eprintln!("  calendar {}: {} component(s)", i, cal.components().len());
    }
    eprintln!("remaining input: {} bytes", input.len());
    eprintln!("load time: {:?}", end - load_start);
    eprintln!("parse time: {:?}", end - parse_start);
}
