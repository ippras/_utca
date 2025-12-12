// use egui::{CollapsingHeader, Grid, Response, Ui, Widget};

// /// `VariableList` renders a nested list of debugger values.
// pub struct VariableList<'a> {
//     values: Box<dyn Iterator<Item = u8> + 'a>,
// }

// impl<'a> VariableList<'a> {
//     pub fn new(values: impl Iterator<Item = u8> + 'a) -> Self {
//         Self {
//             values: Box::new(values),
//         }
//     }
// }

// impl<'a> Widget for VariableList<'a> {
//     fn ui(self, ui: &mut Ui) -> Response {
//         Grid::new(ui.next_auto_id())
//             .num_columns(3)
//             .striped(true)
//             .show(ui, |ui| {
//                 for value in self.values {
//                     if value.children().count() > 0 {
//                         CollapsingHeader::new(value.name().expect("name should be present"))
//                             .id_source(ui.next_auto_id())
//                             .show(ui, |ui| {
//                                 ui.add(VariableList::new(value.children()));
//                             });
//                     } else {
//                         ui.label(value.name().unwrap_or_default());
//                         ui.label(value.display_type_name().unwrap_or_default());
//                         ui.label(value.value().unwrap_or_default());
//                     }
//                     ui.end_row();
//                 }
//             })
//             .response
//     }
// }
