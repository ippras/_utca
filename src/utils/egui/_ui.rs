use egui::{Align, Color32, FontId, TextFormat, TextStyle, Ui, Visuals, text::LayoutJob};

// struct CompositeText {
//     color: Color32,
//     layout_job: LayoutJob,
//     style: Arc<Style>,
// }

/// Extension methods for [`Ui`]
pub trait UiExt {
    fn subscripted_text(
        &self,
        text: &str,
        subscription: &str,
        format: SubscriptedTextFormat,
    ) -> LayoutJob;
}

impl UiExt for Ui {
    fn subscripted_text(
        &self,
        text: &str,
        subscription: &str,
        format: SubscriptedTextFormat,
    ) -> LayoutJob {
        let mut layout_job = LayoutJob::default();
        let color = format.color.unwrap_or_else(|| {
            if format.widget {
                if self.visuals().dark_mode {
                    Visuals::dark().widgets.inactive.text_color()
                } else {
                    Visuals::light().widgets.inactive.text_color()
                }
            } else if self.visuals().dark_mode {
                Visuals::dark().text_color()
            } else {
                Visuals::light().text_color()
            }
        });
        let font_id = format.font_id.unwrap_or_default();
        layout_job.append(
            text,
            0.0,
            TextFormat {
                color,
                font_id,
                ..Default::default()
            },
        );
        let font_id = format
            .small_font_id
            .unwrap_or_else(|| TextStyle::Small.resolve(self.style()));
        layout_job.append(
            subscription,
            1.0,
            TextFormat {
                color,
                font_id,
                valign: Align::BOTTOM,
                ..Default::default()
            },
        );
        layout_job
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubscriptedTextFormat {
    pub color: Option<Color32>,
    pub font_id: Option<FontId>,
    pub small_font_id: Option<FontId>,
    pub widget: bool,
}
