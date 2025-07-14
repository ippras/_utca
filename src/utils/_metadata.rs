use metadata::Metadata;

pub fn title(metadata: &Metadata, separator: &str) -> String {
    let name = &metadata.name;
    match (&metadata.date, &metadata.version) {
        (None, None) => name.to_owned(),
        (None, Some(version)) => format!("{name}{separator}{version}"),
        (Some(date), None) => format!("{name}{separator}{date}"),
        (Some(date), Some(version)) => {
            format!("{name}{separator}{date}{separator}{version}")
        }
    }
}
