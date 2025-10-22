use crate::utils::HashedMetaDataFrame;
use chrono::NaiveDate;
use itertools::Itertools as _;
use metadata::{AUTHORS, DATE, DESCRIPTION, NAME};

const DATE_FORMAT: &str = "%Y-%m-%d";

pub(crate) fn authors(frames: &[HashedMetaDataFrame]) -> String {
    frames
        .iter()
        .flat_map(|frame| frame.meta.get(AUTHORS).map(|authors| authors.split(",")))
        .flatten()
        .unique()
        .join(",")
}

pub(crate) fn date(frames: &[HashedMetaDataFrame]) -> String {
    let mut date = None;
    for frame in frames {
        date = std::cmp::max(
            date,
            frame
                .meta
                .get(DATE)
                .and_then(|date| NaiveDate::parse_from_str(date, DATE_FORMAT).ok()),
        );
    }
    date.map_or_default(|date| date.to_string())
}

pub(crate) fn description(frames: &[HashedMetaDataFrame]) -> String {
    frames
        .iter()
        .flat_map(|frame| frame.meta.get(DESCRIPTION).map(|description| description))
        .unique()
        .join("\n")
}

pub(crate) fn name(frames: &[HashedMetaDataFrame]) -> String {
    format!(
        "[{}]",
        frames
            .iter()
            .filter_map(|frame| frame.meta.get(NAME))
            .unique()
            .format_with(", ", |name, f| f(&name))
    )
}
