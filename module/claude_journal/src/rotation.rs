//! Daily JSONL file rotation — UTC-date-based filename generation.

use chrono::{ Datelike, Utc };

/// Return the JSONL filename for the given UTC year/month/day.
///
/// Format: `YYYY-MM-DD.jsonl`
#[ inline ]
#[ must_use ]
pub fn date_filename( year : i32, month : u32, day : u32 ) -> String
{
  format!( "{:04}-{:02}-{:02}.jsonl", year, month, day )
}

/// Return the JSONL filename for today's UTC date.
///
/// Equivalent to `date_filename` called with the current UTC year/month/day.
#[ inline ]
#[ must_use ]
pub fn today_filename() -> String
{
  let now = Utc::now();
  date_filename( now.year(), now.month(), now.day() )
}
