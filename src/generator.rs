//! The mark rspandoc leaves in every whole document it writes: its name,
//! its version and its author, where each format keeps the name of the
//! program that made the file (pandoc writes `pandoc` in the same places).
//! A fragment is a piece of someone else's document and carries no mark.
//! `--no-mark` leaves the mark out: the document is then pandoc's, down to
//! the name `pandoc`, which is what the comparisons with pandoc run.
//!
//! Two more things are written only when asked for, since neither is the
//! same from one run to the next: when the conversion happened
//! (`--mark-date`) and who ran it (`--mark-by NAME`).

use anyhow::{Context as _, Result, bail};

/// What pandoc writes in those places, and rspandoc with `--no-mark`.
pub const PANDOC: &str = "pandoc";

/// `rspandoc VERSION by AUTHOR`: the authors of `Cargo.toml`, without
/// their addresses.
pub fn generator() -> String {
    let authors: Vec<&str> = env!("CARGO_PKG_AUTHORS")
        .split(':')
        .map(|a| a.split('<').next().unwrap_or(a).trim())
        .collect();
    format!(
        "rspandoc {} by {}",
        env!("CARGO_PKG_VERSION"),
        authors.join(", ")
    )
}

/// What a document says about how it was made.
pub struct Mark {
    /// The program named as the document's generator: rspandoc's mark, or
    /// `pandoc` with `--no-mark`.
    pub generator: String,
    /// Whether that is rspandoc's mark.
    pub named: bool,
    /// When the conversion happened (`--mark-date`), as
    /// `YYYY-MM-DDThh:mm:ssZ`.
    pub date: Option<String>,
    /// Who ran the conversion (`--mark-by`).
    pub by: Option<String>,
}

impl Mark {
    pub fn new(no_mark: bool, date: bool, by: Option<&str>) -> Result<Mark> {
        Ok(Mark {
            generator: if no_mark {
                PANDOC.to_owned()
            } else {
                generator()
            },
            named: !no_mark,
            date: date.then(now).transpose()?,
            by: by.map(str::to_owned),
        })
    }

    /// Whether the mark says more than the program's name.
    pub fn has_details(&self) -> bool {
        self.date.is_some() || self.by.is_some()
    }
}

/// The time of the conversion: `SOURCE_DATE_EPOCH` when it is set, so
/// that a build can be repeated to the byte, and the clock otherwise.
fn now() -> Result<String> {
    let secs = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(text) => match text.trim().parse::<u64>() {
            Ok(secs) => secs,
            Err(_) => bail!("SOURCE_DATE_EPOCH is not a number of seconds: `{text}`"),
        },
        Err(_) => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .context("the clock is before 1970")?
            .as_secs(),
    };
    Ok(utc(secs))
}

/// `secs` since 1970 as `YYYY-MM-DDThh:mm:ssZ` (the W3C date and time
/// format, which is what HTML and DOCX metadata take).
fn utc(secs: u64) -> String {
    let (days, rest) = (secs / 86_400, secs % 86_400);
    // Days to a date in the Gregorian calendar, counting years from 1
    // March so that the leap day is a year's last (Howard Hinnant's
    // `civil_from_days`).
    let z = days + 719_468;
    let (era, day_of_era) = (z / 146_097, z % 146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let m = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * m + 2) / 5 + 1;
    let month = if m < 10 { m + 3 } else { m - 9 };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3_600,
        rest % 3_600 / 60,
        rest % 60
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn names_the_program_the_version_and_the_author() {
        let mark = super::generator();
        assert!(mark.starts_with(concat!("rspandoc ", env!("CARGO_PKG_VERSION"), " by ")));
        assert!(mark.len() > concat!("rspandoc ", env!("CARGO_PKG_VERSION"), " by ").len());
        assert!(!mark.contains('<') && !mark.contains('@'), "{mark}");
    }

    #[test]
    fn dates_are_utc() {
        assert_eq!(super::utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(super::utc(951_782_399), "2000-02-28T23:59:59Z");
        assert_eq!(super::utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(super::utc(1_709_251_199), "2024-02-29T23:59:59Z");
        assert_eq!(super::utc(1_795_168_496), "2026-11-20T09:54:56Z");
        assert_eq!(super::utc(4_102_444_800), "2100-01-01T00:00:00Z");
    }
}
