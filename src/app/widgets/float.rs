use crate::app::ResultExt as _;
use egui::{DragValue, InnerResponse, RichText, Ui, Widget, vec2};
use polars::prelude::*;

/// Float widget
pub(crate) struct FloatWidget<'a> {
    value: Box<dyn Fn() -> PolarsResult<Option<f64>> + 'a>,
    settings: Settings,
}

impl<'a> FloatWidget<'a> {
    pub(crate) fn new(value: impl Fn() -> PolarsResult<Option<f64>> + 'a) -> Self {
        Self {
            value: Box::new(value),
            settings: Settings::default(),
        }
    }

    pub(crate) fn disable(mut self, disable: bool) -> Self {
        self.settings.disable = disable;
        self
    }

    pub(crate) fn editable(mut self, editable: bool) -> Self {
        self.settings.editable = editable;
        self
    }

    pub(crate) fn hover(mut self) -> Self {
        self.settings.hover = true;
        self
    }

    pub(crate) fn percent(mut self, percent: bool) -> Self {
        self.settings.percent = percent;
        self
    }

    pub(crate) fn precision(mut self, precision: Option<usize>) -> Self {
        self.settings.precision = precision;
        self
    }

    pub(crate) fn try_show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<Option<f64>>> {
        let format = |value: f64| match self.settings.precision {
            Some(precision) => format!("{value:.precision$}"),
            None => AnyValue::from(value).to_string(),
        };
        if self.settings.disable {
            ui.disable();
        }
        let mut inner = (self.value)();
        // let Ok(Some(mut value)) = inner else {
        //     // Null
        //     let response = ui.label(AnyValue::Null.to_string());
        //     return InnerResponse::new(inner, response);
        // };
        let Ok(mut value) = inner else {
            let response = ui.label("");
            return InnerResponse::new(inner, response);
        };
        let value = value.get_or_insert_default();
        // let Ok(Some(mut value)) = match inner {
        //     Ok(Some(mut value)) => value,
        //     Ok(None) => ,
        //     // Null
        //     let response = ui.label(AnyValue::Null.to_string());
        //     return InnerResponse::new(inner, response);
        // };
        // Percent
        if self.settings.percent {
            *value *= 100.0;
        }
        // Editable
        let mut response = if self.settings.editable {
            // Writable
            let response = ui.add_sized(
                vec2(ui.available_width(), ui.spacing().interact_size.y),
                DragValue::new(value)
                    .range(0.0..=f64::MAX)
                    .custom_formatter(|value, _| format(value)),
            );
            if response.changed() {
                inner = Ok(Some(*value));
            }
            response
        } else {
            // Readable
            ui.label(format(*value))
        };
        if self.settings.hover {
            response = response.on_hover_text(RichText::new(AnyValue::Float64(*value).to_string()));
        }
        InnerResponse::new(inner, response)
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<Option<f64>> {
        let InnerResponse { inner, response } = self.try_show(ui);
        let inner = inner.context(ui.ctx()).flatten();
        InnerResponse::new(inner, response)
    }
}

impl Widget for FloatWidget<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui).response
    }
}

/// Settings
#[derive(Clone, Copy, Debug, Default)]
struct Settings {
    editable: bool,
    disable: bool,
    percent: bool,
    hover: bool,
    precision: Option<usize>,
}
