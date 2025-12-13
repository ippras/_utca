// /// Column filter
// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub(crate) struct ColumnFilter {
//     pub(crate) columns: Vec<Column>,
// }

// impl ColumnFilter {
//     pub(crate) fn new() -> Self {
//         Self {
//             columns: Vec::new(),
//         }
//     }

//     pub(crate) fn update(&mut self, columns: &[&str]) {
//         let mut has_columns = HashSet::default();
//         for &name in columns {
//             has_columns.insert(name);
//             if !self.columns.iter().any(|column| column.name == name) {
//                 self.columns.push(Column::new(name.to_owned()));
//             }
//         }
//         self.columns
//             .retain(|column| has_columns.contains(&*column.name));
//     }

//     pub(crate) fn iter_visible_columns(&self) -> impl Iterator<Item = &Column> {
//         self.columns.iter().filter(|column| column.visible)
//     }

//     pub(crate) fn iter_visible_column_names(&self) -> impl Iterator<Item = &str> {
//         self.iter_visible_columns()
//             .map(|column| column.name.as_str())
//     }
// }

// impl ColumnFilter {
//     pub(crate) fn show(&mut self, ui: &mut Ui) {
//         self.columns(ui);
//     }

//     pub(crate) fn columns(&mut self, ui: &mut Ui) {
//         let response =
//             dnd(ui, ui.next_auto_id()).show(self.columns.iter_mut(), |ui, item, handle, _state| {
//                 let visible = item.visible;
//                 Sides::new().show(
//                     ui,
//                     |ui| {
//                         handle.ui(ui, |ui| {
//                             ui.label(DOTS_SIX_VERTICAL);
//                         });
//                         let mut label = RichText::new(&item.name);
//                         if !visible {
//                             label = label.weak();
//                         }
//                         ui.label(label);
//                     },
//                     |ui| {
//                         if ui
//                             .small_button(if item.visible { EYE } else { EYE_SLASH })
//                             .clicked()
//                         {
//                             item.visible = !item.visible;
//                         }
//                     },
//                 );
//             });
//         if response.is_drag_finished() {
//             response.update_vec(self.columns.as_mut_slice());
//         }
//     }
// }

// /// Column
// #[derive(Clone, Debug, Deserialize, Hash, Serialize)]
// pub(crate) struct Column {
//     name: String,
//     visible: bool,
// }

// impl Column {
//     pub(crate) fn new(name: String) -> Self {
//         Self {
//             name,
//             visible: true,
//         }
//     }
// }
