use ansi_term::Colour::Green;
use ansi_term::Style;
use chrono::{DateTime, Local, TimeZone, Utc};
use exitfailure::ExitFailure;
use failure::Context;
use failure::Fail;
use failure::ResultExt;
use regex::Regex;
use structopt::clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    enum Fmt {
      RFC2822,
      RFC3399,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Epoch converter", about = "Clone of epochconverter.com")]
struct Cli {
    /// epoch to convert. Uses current unix timestamp by default.
    epoch: Option<i64>,
    /// Human date to convert. RFC 2822, D-M-Y, M/D/Y, Y-M-D, etc. Strip 'GMT' to convert to local.
    ///
    /// Supported input formats:
    /// - RFC 2822: 31 Jan 1970 00:00:00 +0000
    /// - D-M-Y: 31-01-1970 00:00:00 +0000
    /// - M/D/Y: 01/31/1970 00:00:00 +0000
    /// - Y/M/D: 1970/01/31 00:00:00 +0000
    ///
    /// Supported TimeZone formats:
    ///  - GMT
    ///  - Offset from the local time to UTC (with UTC being +0000).
    #[structopt(short, long, conflicts_with("epoch"), parse(try_from_str = parse_datetime), verbatim_doc_comment)]
    datetime: Option<DateTime<Utc>>,
    #[structopt(short, long, possible_values = & Fmt::variants(), case_insensitive = true)]
    output_fmt: Option<Fmt>,
}

#[derive(Fail, Debug)]
#[fail(display = "Missing timezone.")]
struct MissingZone;

fn parse_datetime(src: &str) -> Result<DateTime<Utc>, Context<String>> {
    let processed_str = src.trim().replace("GMT", "+0000");
    let re = Regex::new(r".*+\d{4}$").unwrap();

    if !re.is_match(&processed_str) {
        Err(MissingZone).with_context(|_| format!("Could not parse input {}. Provide valid timezone or GMT as suffix.", processed_str))
    } else {
        DateTime::parse_from_rfc2822(&processed_str)
            .or(DateTime::parse_from_str(&processed_str, "%d-%m-%Y %T %z")
                .or(DateTime::parse_from_str(&processed_str, "%m/%d/%Y %T %z")
                    .or(DateTime::parse_from_str(&processed_str, "%Y/%m/%d %T %z")
                        .or(Err(MissingZone))
                    )
                )
            )
            .map(|dt| dt.with_timezone(&Utc))
            .with_context(|_| format!("could not parse input: {}", processed_str))
    }
}

fn display_results(datetime: &DateTime<Utc>) {
    println!("{}: {}", Style::new().fg(Green).bold().paint("Epoch timestamp"), datetime.timestamp());
    println!("Timestamp in milliseconds: {}", datetime.timestamp() * 1000 + datetime.timestamp_subsec_millis() as i64);

    println!("{}: {:?}", Style::new().fg(Green).bold().paint("Date and time (GMT)"), datetime.to_rfc2822());
    println!("Date and time (your time zone): {}", datetime.with_timezone(&Local).to_rfc2822());
}

fn main() -> Result<(), ExitFailure> {
    let args = Cli::from_args();
    let datetime: DateTime<Utc>;

    if let Some(datetime) = args.datetime {
        display_results(&datetime);
    } else {
        let now = Utc::now();

        if args.epoch.is_some() {
            let ts = now.timestamp();

            let input = args.epoch.unwrap();

            let mut seconds: i64 = 0;
            let mut nano_seconds: u32 = 0;

            let threshold: i64 = 10;
            let milli_multiplier: i64 = 10i64.pow(3);
            let micro_multiplier: i64 = 10i64.pow(6);
            let nano_multiplier: i64 = 10i64.pow(9);

            if input <= ts * threshold {
                println!("Assuming that timestamp is in seconds.");
                seconds = input;
            } else if (input > ts * threshold) && (input <= ts * milli_multiplier * threshold) {
                println!("Assuming that timestamp is in milliseconds.");
                seconds = input / milli_multiplier;
                nano_seconds = (micro_multiplier * (input % milli_multiplier)) as u32;
            } else if input > ts * milli_multiplier * threshold && input <= ts * micro_multiplier * threshold {
                println!("Assuming that timestamp is in microseconds.");
                seconds = input / micro_multiplier;
                nano_seconds = (milli_multiplier * (input % micro_multiplier)) as u32;
            } else if input > ts * micro_multiplier * threshold {
                println!("Assuming that timestamp is in nanoseconds.");
                seconds = ts / nano_multiplier;
                nano_seconds = (ts % nano_multiplier) as u32;
            }

            datetime = Utc.timestamp(seconds, nano_seconds);
        } else {
            datetime = now;
        }
        display_results(&datetime);
    }
    Ok(())
}
