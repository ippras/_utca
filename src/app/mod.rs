use self::{
    data::Data,
    identifiers::{CALCULATE, COMPOSE, CONFIGURE, DATA},
    panes::{Behavior, Pane},
    states::State,
    widgets::{About, Github, Presets},
};
use crate::{
    app::widgets::{
        AboutButton, GridButton, HorizontalButton, LeftPanelButton, ResetButton, SettingsButton,
        TabsButton, VerticalButton,
    },
    localization::ContextExt as _,
    utils::{HashedDataFrame, HashedMetaDataFrame},
};
use anyhow::Result;
use chrono::Local;
use eframe::{APP_KEY, CreationContext, Storage, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, Context, DroppedFile, FontDefinitions, Frame, Id,
    LayerId, Layout, MenuBar, Order, RichText, ScrollArea, SidePanel, Sides, TextStyle,
    TopBottomPanel, Ui, Visuals, Widget as _, Window, warn_if_debug_build,
};
use egui_ext::{DroppedFileExt as _, HoveredFileExt, LightDarkButton};
use egui_l20n::prelude::*;
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{INFO, PLUS, SLIDERS_HORIZONTAL},
};
use egui_tiles::{Tile, Tree};
use egui_tiles_ext::{HORIZONTAL, TreeExt as _, VERTICAL};
use lipid::prelude::*;
use metadata::{DATE, Metadata, NAME, polars::MetaDataFrame};
use panes::configuration::SCHEMA;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{borrow::BorrowMut, fmt::Write, str, sync::LazyLock};
use tracing::{info, instrument, trace};

const ICON_SIZE: f32 = 32.0;
const ID_SOURCE: &str = "UTCA";
/// IEEE 754-2008
const MAX_PRECISION: usize = 16;

// const DESCRIPTION: &str = "Positional-species and positional-type composition of TAG from mature fruit arils of the Euonymus section species, mol % of total TAG";

fn custom_style(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = custom_visuals(style.visuals);
    ctx.set_style(style);
}

fn custom_visuals<T: BorrowMut<Visuals>>(mut visuals: T) -> T {
    visuals.borrow_mut().collapsing_header_frame = true;
    visuals
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    // Panels
    left_panel: bool,
    // Data
    data: Data,
    // Panes
    tree: Tree<Pane>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            left_panel: true,
            tree: Tree::empty("CentralTree"),
            data: Default::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext) -> Self {
        // Customize style of egui.
        let mut fonts = FontDefinitions::default();
        add_to_fonts(&mut fonts, Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_localizations();
        custom_style(&cc.egui_ctx);

        // return Default::default();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        cc.storage
            .and_then(|storage| get_value(storage, APP_KEY))
            .unwrap_or_default()
    }
}

// Panels
impl App {
    fn panels(&mut self, ctx: &Context, state: &mut State) {
        self.top_panel(ctx, state);
        self.bottom_panel(ctx);
        self.left_panel(ctx);
        self.central_panel(ctx);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("BottomPanel").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Sides::new().show(
                    ui,
                    |_| {},
                    |ui| {
                        warn_if_debug_build(ui);
                        ui.label(RichText::new(env!("CARGO_PKG_VERSION")).small());
                        ui.separator();
                    },
                );
            });
        });
    }

    // Central panel
    fn central_panel(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(Frame::central_panel(&ctx.style()).inner_margin(0))
            .show(ctx, |ui| {
                let mut behavior = Behavior { close: None };
                self.tree.ui(&mut behavior, ui);
                if let Some(id) = behavior.close {
                    self.tree.tiles.remove(id);
                }
            });
    }

    // Left panel
    fn left_panel(&mut self, ctx: &Context) {
        SidePanel::left("LeftPanel")
            .resizable(true)
            .show_animated(ctx, self.left_panel, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    self.data.show(ui);
                });
            });
    }

    // Top panel
    fn top_panel(&mut self, ctx: &Context, state: &mut State) {
        TopBottomPanel::top("TopPanel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    LeftPanelButton::new(&mut self.left_panel)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    // Light/Dark
                    ui.light_dark_button(ICON_SIZE);
                    ui.separator();
                    ResetButton::new(&mut state.settings.reset_state)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    self.layouts(ui, state);
                    ui.separator();
                    SettingsButton::new(&mut state.windows.open_settings)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                    // Create
                    if ui
                        .button(RichText::new(PLUS).size(ICON_SIZE))
                        .on_hover_localized("Create")
                        .clicked()
                    {
                        let data = DataFrame::empty_with_schema(&SCHEMA);
                        let mut meta = Metadata::default();
                        meta.insert(NAME.to_owned(), "Untitled".to_owned());
                        meta.insert(DATE.to_owned(), Local::now().date_naive().to_string());
                        self.data.add(MetaDataFrame::new(
                            meta,
                            HashedDataFrame {
                                data_frame: data,
                                hash: 0,
                            },
                        ));
                    }
                    // Presets
                    Presets.ui(ui);
                    ui.separator();
                    Github.ui(ui);
                    ui.separator();
                    AboutButton::new(&mut state.windows.open_about)
                        .size(ICON_SIZE)
                        .ui(ui);
                    ui.separator();
                });
            });
        });
    }

    fn layouts(&mut self, ui: &mut Ui, state: &mut State) {
        VerticalButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        HorizontalButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        GridButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
        TabsButton::new(&mut state.settings.layout.container_kind)
            .size(ICON_SIZE)
            .ui(ui);
    }
}

// Windows
impl App {
    fn windows(&mut self, ctx: &Context, state: &mut State) {
        self.about_window(ctx, state);
        self.settings_window(ctx, state);
    }

    fn about_window(&mut self, ctx: &Context, state: &mut State) {
        Window::new(format!("{INFO} About"))
            .open(&mut state.windows.open_about)
            .show(ctx, |ui| About.ui(ui));
    }

    fn settings_window(&mut self, ctx: &Context, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Settings"))
            .open(&mut state.windows.open_settings)
            .show(ctx, |ui| {
                state.settings.show(ui);
            });
    }
}

// Copy/Paste, Drag&Drop
impl App {
    fn data(&mut self, ctx: &Context) {
        if let Some(frame) =
            ctx.data_mut(|data| data.remove_temp::<HashedMetaDataFrame>(Id::new(DATA)))
        {
            self.data.add(frame);
            self.left_panel = true;
        }
    }

    fn configure(&mut self, ctx: &Context) {
        if let Some(frames) =
            ctx.data_mut(|data| data.remove_temp::<Vec<HashedMetaDataFrame>>(Id::new(CONFIGURE)))
        {
            self.tree
                .insert_pane::<VERTICAL>(Pane::configuration(frames));
        }
    }

    fn calculate(&mut self, ctx: &Context) {
        if let Some(frames) =
            ctx.data_mut(|data| data.remove_temp::<Vec<HashedMetaDataFrame>>(Id::new(CALCULATE)))
        {
            self.tree.insert_pane::<VERTICAL>(Pane::calculation(frames));
        }
    }

    fn compose(&mut self, ctx: &Context) {
        if let Some(frames) =
            ctx.data_mut(|data| data.remove_temp::<Vec<HashedMetaDataFrame>>(Id::new(COMPOSE)))
        {
            self.tree
                .insert_pane::<HORIZONTAL>(Pane::composition(frames));
        } else if let Some(frame) =
            ctx.data_mut(|data| data.remove_temp::<MetaDataFrame>(Id::new(COMPOSE)))
        {
            tracing::error!("COMPOSE");
            // self.tree
            //     .insert_pane::<HORIZONTAL>(Pane::composition(frame));
        }
    }

    fn drag_and_drop(&mut self, ctx: &Context) {
        // Preview hovering files
        if let Some(text) = ctx.input(|input| {
            (!input.raw.hovered_files.is_empty()).then(|| {
                let mut text = String::from("Dropping files:");
                for file in &input.raw.hovered_files {
                    write!(text, "\n{}", file.display()).ok();
                }
                text
            })
        }) {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let content_rect = ctx.content_rect();
            painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
        // Parse dropped files
        if let Some(dropped_files) = ctx.input(|input| {
            (!input.raw.dropped_files.is_empty()).then_some(input.raw.dropped_files.clone())
        }) {
            info!(?dropped_files);
            for dropped_file in dropped_files {
                _ = self.parse(ctx, dropped_file);
            }
        }
    }

    #[instrument(skip_all, err)]
    fn parse(&mut self, ctx: &Context, dropped_file: DroppedFile) -> Result<()> {
        const CONFIGURATION: LazyLock<SchemaRef> = LazyLock::new(|| {
            Arc::new(Schema::from_iter([
                Field::new(PlSmallStr::from_static(LABEL), DataType::String),
                field!(FATTY_ACID),
                Field::new(
                    PlSmallStr::from_static("Triacylglycerol"),
                    DataType::Float64,
                ),
                Field::new(
                    PlSmallStr::from_static("Diacylglycerol1223"),
                    DataType::Float64,
                ),
                Field::new(
                    PlSmallStr::from_static("Monoacylglycerol2"),
                    DataType::Float64,
                ),
            ]))
        });

        const COMPOSITION: LazyLock<SchemaRef> = LazyLock::new(|| {
            Arc::new(Schema::from_iter([
                field!(LABEL[DataType::String]),
                field!(TRIACYLGLYCEROL[data_type!(FATTY_ACID)]),
                Field::new(PlSmallStr::from_static("Value"), DataType::Float64),
            ]))
        });

        /// Monoacylglycerol
        const MAG: LazyLock<SchemaRef> = LazyLock::new(|| {
            Arc::new(Schema::from_iter([
                Field::new(PlSmallStr::from_static(LABEL), DataType::String),
                field!(FATTY_ACID),
                Field::new(
                    PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS123),
                    DataType::Float64,
                ),
                Field::new(
                    PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS2),
                    DataType::Float64,
                ),
            ]))
        });

        /// Diacylglycerol
        const DAG: LazyLock<SchemaRef> = LazyLock::new(|| {
            Arc::new(Schema::from_iter([
                Field::new(PlSmallStr::from_static(LABEL), DataType::String),
                field!(FATTY_ACID),
                Field::new(
                    PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS123),
                    DataType::Float64,
                ),
                Field::new(
                    PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS12_23),
                    DataType::Float64,
                ),
            ]))
        });

        let bytes = dropped_file.bytes()?;
        trace!(?bytes);
        // let reader = Cursor::new(bytes);
        // let mut reader = ParquetReader::new(reader).set_rechunk(true);
        // let meta: Metadata = reader
        //     .get_metadata()?
        //     .key_value_metadata()
        //     .as_ref()
        //     .map(|key_values| {
        //         key_values
        //             .into_iter()
        //             .filter_map(|KeyValue { key, value }| {
        //                 if key != "ARROW:schema" {
        //                     Some((key.to_upper_camel_case(), value.clone()?))
        //                 } else {
        //                     None
        //                 }
        //             })
        //             .collect()
        //     })
        //     .unwrap_or_default();
        // println!("meta: {meta:?}");
        // let data = reader.finish()?;
        // let frame = MetaDataFrame { meta, data };
        // let name = format!("{}.utca.ron", frame.meta.format("."));
        // println!("name: {name}");
        // _ = export::ron::save(&frame, &name);
        // return Ok(());

        let frame = ron::de::from_bytes::<MetaDataFrame>(&bytes)?;
        let hashed_frame = MetaDataFrame {
            meta: frame.meta,
            data: HashedDataFrame::new(frame.data)?,
        };
        let schema = hashed_frame.data.schema();
        if CONFIGURATION.matches_schema(schema).is_ok_and(|cast| !cast) {
            info!("CONFIGURATION");
            self.data.add(hashed_frame);
        } else if COMPOSITION.matches_schema(schema).is_ok_and(|cast| !cast) {
            info!("COMPOSITION");
            ctx.data_mut(|data| data.insert_temp(Id::new(COMPOSE), hashed_frame));
        } else if MAG.ensure_is_exact_match(schema).is_ok() {
            info!(STEREOSPECIFIC_NUMBERS2);
            self.data.add(hashed_frame);
        } else if DAG.ensure_is_exact_match(schema).is_ok() {
            info!(STEREOSPECIFIC_NUMBERS2);
            self.data.add(hashed_frame);
        } else {
            return Err(
                polars_err!(SchemaMismatch: r#"Invalid dropped file schema: expected [`CONFIGURATION`, `COMPOSITION`], got = `{schema:?}`"#),
            )?;
        }
        Ok(())
    }

    // fn paste(&mut self, ctx: &Context) {
    //     if !ctx.memory(|memory| memory.focused().is_some()) {
    //         ctx.input(|input| {
    //             for event in &input.raw.events {
    //                 if let Event::Paste(text) = event {
    //                     println!("Paste: {text}");
    //                     // ctx.data_mut(|data| {
    //                     //     data.insert_temp(Id::new("Paste"), text.clone());
    //                     // });
    //                 }
    //             }
    //         });
    //     }
    // }

    // fn parse(&mut self, paste: &str) -> Result<()> {
    //     use crate::parsers::whitespace::Parser;
    //     let parsed = Parser::parse(paste)?;
    //     debug!(?parsed);
    //     for parsed in parsed {
    //         // self.docks.central.tabs.input.add(match parsed {
    //         //     Parsed::All(label, (c, n), tag, dag, mag) => FattyAcid {
    //         //         label,
    //         //         formula: ether!(c as usize, n as usize),
    //         //         values: [tag, dag, mag],
    //         //     },
    //         //     // Parsed::String(label) => Row { label, ..default() },
    //         //     // Parsed::Integers(_) => Row { label, ..default() },
    //         //     // Parsed::Float(tag) => Row { label, ..default() },
    //         //     _ => unimplemented!(),
    //         // })?;
    //         // self.config.push_row(Row {
    //         //     acylglycerols,
    //         //     label:  parsed.,
    //         //     ether: todo!(),
    //         //     // ..default()
    //         // })?;
    //     }
    //     // let mut rows = Vec::new();
    //     // for row in paste.split('\n') {
    //     //     let mut columns = [0.0; COUNT];
    //     //     for (j, column) in row.split('\t').enumerate() {
    //     //         ensure!(j < COUNT, "Invalid shape, columns: {COUNT} {j}");
    //     //         columns[j] = column.replace(',', ".").parse()?;
    //     //     }
    //     //     rows.push(columns);
    //     // }
    //     // for acylglycerols in rows {
    //     //     self.config.push_row(Row {
    //     //         acylglycerol: acylglycerols,
    //     //         ..default()
    //     //     })?;
    //     // }
    //     Ok(())
    // }

    // fn export(&self) -> Result<(), impl Debug> {
    //     let content = to_string(&TomlParsed {
    //         name: self.context.state.entry().meta.name.clone(),
    //         fatty_acids: self.context.state.entry().fatty_acids(),
    //     })
    //     .unwrap();
    //     self.file_dialog
    //         .save(
    //             &format!("{}.toml", self.context.state.entry().meta.name),
    //             content,
    //         )
    //         .unwrap();
    //     Ok::<_, ()>(())
    // }

    // fn import(&mut self) -> Result<(), impl Debug> {
    //     self.file_dialog.load()
    // }

    fn state(&mut self, ctx: &Context, state: &mut State) {
        if state.settings.reset_state {
            *self = Default::default();
            // Cache
            let caches = ctx.memory_mut(|memory| memory.caches.clone());
            ctx.memory_mut(|memory| {
                memory.caches = caches;
            });
            ctx.set_localizations();
            state.settings.reset_state = false;
        }
        if let Some(container_kind) = state.settings.layout.container_kind.take()
            && let Some(id) = self.tree.root
            && let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id)
        {
            container.set_kind(container_kind);
        }
        // if state.settings.reactive {
        //     ctx.request_repaint();
        // }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per
    /// second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut state = State::load(ctx, Id::new(ID_SOURCE));
        self.data(ctx);
        self.configure(ctx);
        self.calculate(ctx);
        self.compose(ctx);
        // Pre update
        self.panels(ctx, &mut state);
        self.windows(ctx, &mut state);
        // Post update
        self.drag_and_drop(ctx);
        self.state(ctx, &mut state);
        state.store(ctx, Id::new(ID_SOURCE));
    }
}

mod computers;
mod data;
mod identifiers;
mod panes;
mod states;
mod widgets;
