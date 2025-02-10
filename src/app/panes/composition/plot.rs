use super::{ID_SOURCE, Settings, State};
use egui::{Align2, Color32, Id, Ui, Vec2b};
use egui_plot::{AxisHints, Bar, BarChart, Line, Plot, PlotPoints};
use polars::prelude::*;
// use super::settings::MC;

/// Composition plot
#[derive(Debug)]
pub(crate) struct PlotView<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
    pub(crate) state: &'a mut State,
}

impl<'a> PlotView<'a> {
    pub const fn new(
        data_frame: &'a DataFrame,
        settings: &'a Settings,
        state: &'a mut State,
    ) -> Self {
        Self {
            data_frame,
            settings,
            state,
        }
    }
}

impl PlotView<'_> {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        self.try_show(ui).unwrap()
    }

    pub(crate) fn try_show(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        // println!("self: {}", self.data_frame.unnest(["Keys"]).unwrap());
        let id_salt = Id::new(ID_SOURCE).with("Plot");
        let mut plot = Plot::new(id_salt)
            .allow_drag(self.state.allow_drag)
            .allow_scroll(self.state.allow_scroll)
            .custom_x_axes(vec![AxisHints::new_x().label("Composition")]);
        if self.state.show_legend {
            plot = plot.legend(Default::default());
        }
        plot.show(ui, |plot_ui| -> PolarsResult<()> {
            let indices = &self.data_frame["Index"];
            let keys = self.data_frame["Keys"].struct_()?;
            let values = self.data_frame["Values"].array()?;
            let selections = &self.settings.confirmed.selections;
            let index = selections.len() - 1;
            let fields = &keys.fields_as_series();
            let keys = &fields[index];
            let mut bars = Vec::new();
            for (row, values) in values.into_iter().enumerate() {
                let values = values.unwrap();
                let mut value = values.f64()?.get(index).unwrap();
                let key = keys.str_value(row)?;
                let x = match self.settings.confirmed.selections[index].composition {
                    MC => keys.f64()?.get(row).unwrap(),
                    EC => keys.i64()?.get(row).unwrap() as _,
                    UC => keys.i64()?.get(row).unwrap() as _,
                    _ => indices.u32()?.get(row).unwrap() as _,
                };
                if self.settings.percent {
                    value *= 100.0;
                }
                bars.push(Bar::new(x, value).name(key));
            }
            plot_ui.bar_chart(BarChart::new(bars).name(index));
            Ok(())
        });
        Ok(())

        // let Self { context } = self;
        // let p = context.settings.visualization.precision;
        // let percent = context.settings.visualization.percent;
        // match context.settings.visualization.source {
        //     Source::Composition => {
        //         context.compose(ui);
        //         let visualized = ui.memory_mut(|memory| {
        //             memory.caches.cache::<Visualized>().get((&*context).into())
        //         });
        //         ui.vertical_centered_justified(|ui| {
        //             let entry = context.state.entry();
        //             ui.heading(&entry.meta.name);
        //             let mut plot = Plot::new("plot")
        //                 .allow_drag(context.settings.visualization.drag)
        //                 .allow_scroll(context.settings.visualization.scroll)
        //                 .y_axis_formatter(move |y, _, _| {
        //                     let rounded = round_to_decimals(y.value, 5).to_string();
        //                     if percent {
        //                         format!("{rounded}%")
        //                     } else {
        //                         format!("{rounded}")
        //                     }
        //                 });
        //             if context.settings.visualization.legend {
        //                 plot = plot.legend(Default::default());
        //             }
        //             plot.show(ui, |ui| {
        //                 // let mut offsets = HashMap::new();
        //                 for (key, values) in visualized {
        //                     // Bars
        //                     let mut offset = 0.0;
        //                     let x = key.into_inner();
        //                     for (name, value) in values {
        //                         let mut y = value;
        //                         if percent {
        //                             y *= 100.0;
        //                         }
        //                         let bar = Bar::new(x, y).name(name).base_offset(offset);
        //                         let chart = BarChart::new(vec![bar])
        //                             .width(context.settings.visualization.width)
        //                             .name(x)
        //                             .color(color(x as _));
        //                         ui.bar_chart(chart);
        //                         offset += y;
        //                     }
        //                     // Text
        //                     if context.settings.visualization.text.show
        //                         && offset >= context.settings.visualization.text.min
        //                     {
        //                         let y = offset;
        //                         let text = Text::new(
        //                             PlotPoint::new(x, y),
        //                             RichText::new(format!("{y:.p$}"))
        //                                 .size(context.settings.visualization.text.size)
        //                                 .heading(),
        //                         )
        //                         .name(x)
        //                         .color(color(x as _))
        //                         .anchor(Align2::CENTER_BOTTOM);
        //                         ui.text(text);
        //                     }
        //                 }
        //             });
        //         });
        //     }
        //     Source::Comparison => {
        //         match context.settings.visualization.comparison {
        //             Comparison::One => {
        //                 context.compare(ui);
        //                 ui.vertical_centered_justified(|ui| {
        //                     let entry = context.state.entry();
        //                     ui.heading(&entry.meta.name);
        //                     let mut plot = Plot::new(ui.id())
        //                         .allow_drag(context.settings.visualization.drag)
        //                         .allow_scroll(context.settings.visualization.scroll);
        //                     if context.settings.visualization.legend {
        //                         plot = plot.legend(Default::default());
        //                     }
        //                     let base: HashMap<_, _> = entry
        //                         .data
        //                         .composed
        //                         .composition(context.settings.composition.method)
        //                         .leaves()
        //                         .map(|Leaf { data }| (data.tag, data.value))
        //                         .collect();
        //                     plot.show(ui, |ui| {
        //                         for (index, entry) in context
        //                             .state
        //                             .entries
        //                             .iter()
        //                             .enumerate()
        //                             .filter(|&(index, _)| index != context.state.index)
        //                         {
        //                             let mut bars = Vec::new();
        //                             let mut offsets = HashMap::new();
        //                             for Hierarchized(_, item) in entry
        //                                 .data
        //                                 .composed
        //                                 .composition(context.settings.composition.method)
        //                                 .hierarchy()
        //                             {
        //                                 match item {
        //                                     Item::Meta(meta) => {}
        //                                     Item::Data(data) => {
        //                                         let name = context.species(data.tag);
        //                                         let ecn = context.ecn(data.tag).sum();
        //                                         let x = ecn as f64;
        //                                         let mut y = base
        //                                             .get(&data.tag)
        //                                             .map_or(0.0, |value| value.0)
        //                                             - data.value.0;
        //                                         if context.settings.visualization.percent {
        //                                             y *= 100.0;
        //                                         }
        //                                         let key = if y < 0.0 {
        //                                             Offset::Negative(ecn)
        //                                         } else {
        //                                             Offset::Positive(ecn)
        //                                         };
        //                                         let offset = offsets.entry(key).or_default();
        //                                         let bar =
        //                                             Bar::new(x, y).name(name).base_offset(*offset);
        //                                         bars.push(bar);
        //                                         *offset += y;
        //                                     }
        //                                 }
        //                             }
        //                             let chart = BarChart::new(bars)
        //                                 .width(context.settings.visualization.width)
        //                                 .name(&entry.meta.name)
        //                                 .color(color(index));
        //                             ui.bar_chart(chart);
        //                             // // Text
        //                             // for (ecn, y) in offsets {
        //                             //     let x = ecn as f64;
        //                             //     let text = Text::new(
        //                             //         PlotPoint::new(x, y),
        //                             //         RichText::new(format!("{y:.p$}")).heading(),
        //                             //     )
        //                             //     .color(color(ecn))
        //                             //     .name(ecn)
        //                             //     .anchor(Align2::CENTER_BOTTOM);
        //                             //     ui.text(text);
        //                             // }
        //                         }
        //                     });
        //                 });
        //             }
        //             Comparison::Many => {
        //                 context.compare(ui);
        //                 // Plot::new("left-top")
        //                 //     .data_aspect(1.0)
        //                 //     .width(250.0)
        //                 //     .height(250.0)
        //                 //     .link_axis(link_group_id, self.link_x, self.link_y)
        //                 //     .link_cursor(link_group_id, self.link_cursor_x, self.link_cursor_y)
        //                 //     .show(ui, LinkedAxesDemo::configure_plot);
        //                 let height = ui.available_height() / context.settings.visualization.height;
        //                 let group_id = ui.id().with("link");
        //                 ui.vertical_centered_justified(|ui| {
        //                     for (index, entry) in context.state.entries.iter().enumerate() {
        //                         ui.heading(&entry.meta.name);
        //                         let mut plot = Plot::new(ui.id().with(index))
        //                             .height(height)
        //                             .allow_drag(context.settings.visualization.drag)
        //                             .allow_scroll(context.settings.visualization.scroll)
        //                             .link_axis(
        //                                 group_id,
        //                                 context.settings.visualization.links.axis.x,
        //                                 context.settings.visualization.links.axis.y,
        //                             )
        //                             .link_cursor(
        //                                 group_id,
        //                                 context.settings.visualization.links.cursor.x,
        //                                 context.settings.visualization.links.cursor.y,
        //                             )
        //                             .set_margin_fraction(Vec2::new(0.0, 0.15));
        //                         if context.settings.visualization.legend {
        //                             plot = plot.legend(Default::default());
        //                         }
        //                         let mut min = [0.0, 0.0];
        //                         let mut max = [0.0, 0.0];
        //                         plot.show(ui, |ui| {
        //                             let mut offsets = HashMap::new();
        //                             for Hierarchized(_, item) in entry
        //                                 .data
        //                                 .composed
        //                                 .composition(context.settings.composition.method)
        //                                 .hierarchy()
        //                             {
        //                                 match item {
        //                                     Item::Meta(meta) => {}
        //                                     Item::Data(data) => {
        //                                         let name = context.species(data.tag);
        //                                         let ecn = context.ecn(data.tag).sum();
        //                                         let x = ecn as f64;
        //                                         min[0] = x.min(min[0]);
        //                                         max[0] = x.max(max[0]);
        //                                         let mut y = data.value.0;
        //                                         if context.settings.visualization.percent {
        //                                             y *= 100.0;
        //                                         }
        //                                         let offset = offsets.entry(ecn).or_default();
        //                                         let bar =
        //                                             Bar::new(x, y).name(name).base_offset(*offset);
        //                                         let chart = BarChart::new(vec![bar])
        //                                             .width(context.settings.visualization.width)
        //                                             .name(ecn)
        //                                             .color(color(ecn));
        //                                         ui.bar_chart(chart);
        //                                         *offset += y;
        //                                     }
        //                                 }
        //                             }
        //                             // Text
        //                             for (ecn, y) in offsets {
        //                                 let x = ecn as f64;
        //                                 let text = Text::new(
        //                                     PlotPoint::new(x, y),
        //                                     RichText::new(format!("{y:.p$}"))
        //                                         .size(context.settings.visualization.text.size)
        //                                         .heading(),
        //                                 )
        //                                 .color(color(ecn))
        //                                 .name(ecn)
        //                                 .anchor(Align2::CENTER_BOTTOM);
        //                                 ui.text(text);
        //                             }
        //                             // ui.set_plot_bounds(PlotBounds::from_min_max(
        //                             //     [33.0, 0.0],
        //                             //     [51.0, 0.0],
        //                             // ));
        //                             // ui.set_auto_bounds(Vec2b::new(false, true));
        //                         });
        //                     }
        //                 });
        //             }
        //         }
        //     }
        // }
    }
}

//     pub(crate) fn ui(&mut self, ui: &mut Ui) {
//         // let Self { context } = self;
//         // let p = context.settings.visualization.precision;
//         // let percent = context.settings.visualization.percent;
//         // match context.settings.visualization.source {
//         //     Source::Composition => {
//         //         context.compose(ui);
//         //         let visualized = ui.memory_mut(|memory| {
//         //             memory.caches.cache::<Visualized>().get((&*context).into())
//         //         });
//         //         ui.vertical_centered_justified(|ui| {
//         //             let entry = context.state.entry();
//         //             ui.heading(&entry.meta.name);
//         //             let mut plot = Plot::new("plot")
//         //                 .allow_drag(context.settings.visualization.drag)
//         //                 .allow_scroll(context.settings.visualization.scroll)
//         //                 .y_axis_formatter(move |y, _, _| {
//         //                     let rounded = round_to_decimals(y.value, 5).to_string();
//         //                     if percent {
//         //                         format!("{rounded}%")
//         //                     } else {
//         //                         format!("{rounded}")
//         //                     }
//         //                 });
//         //             if context.settings.visualization.legend {
//         //                 plot = plot.legend(Default::default());
//         //             }
//         //             plot.show(ui, |ui| {
//         //                 // let mut offsets = HashMap::new();
//         //                 for (key, values) in visualized {
//         //                     // Bars
//         //                     let mut offset = 0.0;
//         //                     let x = key.into_inner();
//         //                     for (name, value) in values {
//         //                         let mut y = value;
//         //                         if percent {
//         //                             y *= 100.0;
//         //                         }
//         //                         let bar = Bar::new(x, y).name(name).base_offset(offset);
//         //                         let chart = BarChart::new(vec![bar])
//         //                             .width(context.settings.visualization.width)
//         //                             .name(x)
//         //                             .color(color(x as _));
//         //                         ui.bar_chart(chart);
//         //                         offset += y;
//         //                     }
//         //                     // Text
//         //                     if context.settings.visualization.text.show
//         //                         && offset >= context.settings.visualization.text.min
//         //                     {
//         //                         let y = offset;
//         //                         let text = Text::new(
//         //                             PlotPoint::new(x, y),
//         //                             RichText::new(format!("{y:.p$}"))
//         //                                 .size(context.settings.visualization.text.size)
//         //                                 .heading(),
//         //                         )
//         //                         .name(x)
//         //                         .color(color(x as _))
//         //                         .anchor(Align2::CENTER_BOTTOM);
//         //                         ui.text(text);
//         //                     }
//         //                 }
//         //             });
//         //         });
//         //     }
//         //     Source::Comparison => {
//         //         match context.settings.visualization.comparison {
//         //             Comparison::One => {
//         //                 context.compare(ui);
//         //                 ui.vertical_centered_justified(|ui| {
//         //                     let entry = context.state.entry();
//         //                     ui.heading(&entry.meta.name);
//         //                     let mut plot = Plot::new(ui.id())
//         //                         .allow_drag(context.settings.visualization.drag)
//         //                         .allow_scroll(context.settings.visualization.scroll);
//         //                     if context.settings.visualization.legend {
//         //                         plot = plot.legend(Default::default());
//         //                     }
//         //                     let base: HashMap<_, _> = entry
//         //                         .data
//         //                         .composed
//         //                         .composition(context.settings.composition.method)
//         //                         .leaves()
//         //                         .map(|Leaf { data }| (data.tag, data.value))
//         //                         .collect();
//         //                     plot.show(ui, |ui| {
//         //                         for (index, entry) in context
//         //                             .state
//         //                             .entries
//         //                             .iter()
//         //                             .enumerate()
//         //                             .filter(|&(index, _)| index != context.state.index)
//         //                         {
//         //                             let mut bars = Vec::new();
//         //                             let mut offsets = HashMap::new();
//         //                             for Hierarchized(_, item) in entry
//         //                                 .data
//         //                                 .composed
//         //                                 .composition(context.settings.composition.method)
//         //                                 .hierarchy()
//         //                             {
//         //                                 match item {
//         //                                     Item::Meta(meta) => {}
//         //                                     Item::Data(data) => {
//         //                                         let name = context.species(data.tag);
//         //                                         let ecn = context.ecn(data.tag).sum();
//         //                                         let x = ecn as f64;
//         //                                         let mut y = base
//         //                                             .get(&data.tag)
//         //                                             .map_or(0.0, |value| value.0)
//         //                                             - data.value.0;
//         //                                         if context.settings.visualization.percent {
//         //                                             y *= 100.0;
//         //                                         }
//         //                                         let key = if y < 0.0 {
//         //                                             Offset::Negative(ecn)
//         //                                         } else {
//         //                                             Offset::Positive(ecn)
//         //                                         };
//         //                                         let offset = offsets.entry(key).or_default();
//         //                                         let bar =
//         //                                             Bar::new(x, y).name(name).base_offset(*offset);
//         //                                         bars.push(bar);
//         //                                         *offset += y;
//         //                                     }
//         //                                 }
//         //                             }
//         //                             let chart = BarChart::new(bars)
//         //                                 .width(context.settings.visualization.width)
//         //                                 .name(&entry.meta.name)
//         //                                 .color(color(index));
//         //                             ui.bar_chart(chart);
//         //                             // // Text
//         //                             // for (ecn, y) in offsets {
//         //                             //     let x = ecn as f64;
//         //                             //     let text = Text::new(
//         //                             //         PlotPoint::new(x, y),
//         //                             //         RichText::new(format!("{y:.p$}")).heading(),
//         //                             //     )
//         //                             //     .color(color(ecn))
//         //                             //     .name(ecn)
//         //                             //     .anchor(Align2::CENTER_BOTTOM);
//         //                             //     ui.text(text);
//         //                             // }
//         //                         }
//         //                     });
//         //                 });
//         //             }
//         //             Comparison::Many => {
//         //                 context.compare(ui);
//         //                 // Plot::new("left-top")
//         //                 //     .data_aspect(1.0)
//         //                 //     .width(250.0)
//         //                 //     .height(250.0)
//         //                 //     .link_axis(link_group_id, self.link_x, self.link_y)
//         //                 //     .link_cursor(link_group_id, self.link_cursor_x, self.link_cursor_y)
//         //                 //     .show(ui, LinkedAxesDemo::configure_plot);
//         //                 let height = ui.available_height() / context.settings.visualization.height;
//         //                 let group_id = ui.id().with("link");
//         //                 ui.vertical_centered_justified(|ui| {
//         //                     for (index, entry) in context.state.entries.iter().enumerate() {
//         //                         ui.heading(&entry.meta.name);
//         //                         let mut plot = Plot::new(ui.id().with(index))
//         //                             .height(height)
//         //                             .allow_drag(context.settings.visualization.drag)
//         //                             .allow_scroll(context.settings.visualization.scroll)
//         //                             .link_axis(
//         //                                 group_id,
//         //                                 context.settings.visualization.links.axis.x,
//         //                                 context.settings.visualization.links.axis.y,
//         //                             )
//         //                             .link_cursor(
//         //                                 group_id,
//         //                                 context.settings.visualization.links.cursor.x,
//         //                                 context.settings.visualization.links.cursor.y,
//         //                             )
//         //                             .set_margin_fraction(Vec2::new(0.0, 0.15));
//         //                         if context.settings.visualization.legend {
//         //                             plot = plot.legend(Default::default());
//         //                         }
//         //                         let mut min = [0.0, 0.0];
//         //                         let mut max = [0.0, 0.0];
//         //                         plot.show(ui, |ui| {
//         //                             let mut offsets = HashMap::new();
//         //                             for Hierarchized(_, item) in entry
//         //                                 .data
//         //                                 .composed
//         //                                 .composition(context.settings.composition.method)
//         //                                 .hierarchy()
//         //                             {
//         //                                 match item {
//         //                                     Item::Meta(meta) => {}
//         //                                     Item::Data(data) => {
//         //                                         let name = context.species(data.tag);
//         //                                         let ecn = context.ecn(data.tag).sum();
//         //                                         let x = ecn as f64;
//         //                                         min[0] = x.min(min[0]);
//         //                                         max[0] = x.max(max[0]);
//         //                                         let mut y = data.value.0;
//         //                                         if context.settings.visualization.percent {
//         //                                             y *= 100.0;
//         //                                         }
//         //                                         let offset = offsets.entry(ecn).or_default();
//         //                                         let bar =
//         //                                             Bar::new(x, y).name(name).base_offset(*offset);
//         //                                         let chart = BarChart::new(vec![bar])
//         //                                             .width(context.settings.visualization.width)
//         //                                             .name(ecn)
//         //                                             .color(color(ecn));
//         //                                         ui.bar_chart(chart);
//         //                                         *offset += y;
//         //                                     }
//         //                                 }
//         //                             }
//         //                             // Text
//         //                             for (ecn, y) in offsets {
//         //                                 let x = ecn as f64;
//         //                                 let text = Text::new(
//         //                                     PlotPoint::new(x, y),
//         //                                     RichText::new(format!("{y:.p$}"))
//         //                                         .size(context.settings.visualization.text.size)
//         //                                         .heading(),
//         //                                 )
//         //                                 .color(color(ecn))
//         //                                 .name(ecn)
//         //                                 .anchor(Align2::CENTER_BOTTOM);
//         //                                 ui.text(text);
//         //                             }
//         //                             // ui.set_plot_bounds(PlotBounds::from_min_max(
//         //                             //     [33.0, 0.0],
//         //                             //     [51.0, 0.0],
//         //                             // ));
//         //                             // ui.set_auto_bounds(Vec2b::new(false, true));
//         //                         });
//         //                     }
//         //                 });
//         //             }
//         //         }
//         //     }
//         // }
//     }
