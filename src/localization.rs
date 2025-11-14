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
    use crate::asset;

    pub(super) const EN: &[&str] = &[
        // asset!("/ftl/en/fatty_acids/byrdwell.com.ftl"),
        asset!("/ftl/en/main.ftl"),
        asset!("/ftl/en/main.ext.ftl"),
        asset!("/ftl/en/fatty_acids/aocs.org.ftl"),
        asset!("/ftl/en/fatty_acids/aocs.org.ext.ftl"),
        asset!("/ftl/en/headers.ftl"),
        asset!("/ftl/en/indices.ftl"),
        asset!("/ftl/en/menu.ftl"),
        asset!("/ftl/en/names.ftl"),
        asset!("/ftl/en/properties.ftl"),
    ];

    pub(super) const RU: &[&str] = &[
        // asset!("/ftl/en/fatty_acids/byrdwell.com.ftl"),
        // asset!("/ftl/en/fatty_acids/ippras.ftl"),
        asset!("/ftl/en/fatty_acids/aocs.org.ftl"),
        asset!("/ftl/ru/headers.ftl"),
        asset!("/ftl/ru/menu.ftl"),
        asset!("/ftl/ru/names.ftl"),
        asset!("/ftl/ru/properties.ftl"),
        asset!("/ftl/ru/settings.ftl"),
    ];
}
