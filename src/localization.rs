use egui::Context;
use egui_l20n::{ContextExt as _, Localization};

/// Extension methods for [`Context`]
pub(crate) trait ContextExt {
    fn set_localizations(&self);
}

impl ContextExt for Context {
    fn set_localizations(&self) {
        self.set_localization(
            locales::EN,
            Localization::new(locales::EN).with_sources(sources::EN),
        );
        self.set_localization(
            locales::RU,
            Localization::new(locales::RU).with_sources(sources::RU),
        );
        self.set_language_identifier(locales::EN)
    }
}

mod locales {
    use egui_l20n::{LanguageIdentifier, langid};

    pub(super) const EN: LanguageIdentifier = langid!("en");
    pub(super) const RU: LanguageIdentifier = langid!("ru");
}

mod sources {
    macro_rules! source {
        ($path:literal) => {
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $path))
        };
    }

    pub(super) const EN: &[&str] = &[
        source!("/ftl/en/fatty_acids/byrdwell.com.ftl"),
        source!("/ftl/en/compositions.ftl"),
        source!("/ftl/en/headers.ftl"),
        source!("/ftl/en/menu.ftl"),
        source!("/ftl/en/names.ftl"),
        source!("/ftl/en/properties.ftl"),
        source!("/ftl/en/settings.ftl"),
    ];

    pub(super) const RU: &[&str] = &[
        source!("/ftl/en/fatty_acids/byrdwell.com.ftl"),
        source!("/ftl/ru/headers.ftl"),
        source!("/ftl/ru/menu.ftl"),
        source!("/ftl/ru/names.ftl"),
        source!("/ftl/ru/properties.ftl"),
        source!("/ftl/ru/settings.ftl"),
    ];
}
