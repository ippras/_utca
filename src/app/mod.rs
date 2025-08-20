use self::{
    data::Data,
    identifiers::{CALCULATE, COMPOSE, CONFIGURE, DATA, GITHUB_TOKEN},
    panes::{Pane, behavior::Behavior},
    widgets::Presets,
    windows::{About, GithubWindow},
};
use crate::localization::ContextExt as _;
use anyhow::Result;
use chrono::Local;
use eframe::{APP_KEY, CreationContext, Storage, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, Context, DroppedFile, FontDefinitions, Frame, Id,
    LayerId, Layout, MenuBar, Order, RichText, ScrollArea, SidePanel, Sides, TextStyle,
    TopBottomPanel, Visuals, util::IdTypeMap, warn_if_debug_build,
};
use egui_ext::{DroppedFileExt as _, HoveredFileExt, LightDarkButton};
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{
        ARROWS_CLOCKWISE, CLOUD_ARROW_DOWN, GEAR, GRID_FOUR, INFO, PLUS, SIDEBAR_SIMPLE,
        SQUARE_SPLIT_HORIZONTAL, SQUARE_SPLIT_VERTICAL, TABS, TRASH,
    },
};
use egui_tiles::{ContainerKind, Tile, Tree};
use egui_tiles_ext::{HORIZONTAL, TreeExt as _, VERTICAL};
use metadata::{DATE, MetaDataFrame, Metadata, NAME};
use panes::configuration::SCHEMA;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{borrow::BorrowMut, fmt::Write, io::Cursor, str};
use tracing::{info, instrument, trace};
use windows::SettingsWindow;

/// IEEE 754-2008
const MAX_PRECISION: usize = 16;
const ICON_SIZE: f32 = 32.0;

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
    // Panes
    tree: Tree<Pane>,
    // Data
    data: Data,

    // Windows
    #[serde(skip)]
    about: About,
    github: GithubWindow,
    settings: SettingsWindow,
}

impl Default for App {
    fn default() -> Self {
        Self {
            left_panel: true,
            tree: Tree::empty("CentralTree"),
            data: Default::default(),
            about: Default::default(),
            github: Default::default(),
            settings: SettingsWindow::default(),
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
        let app = Self::load(cc).unwrap_or_default();
        app
    }

    fn load(cc: &CreationContext) -> Option<Self> {
        let storage = cc.storage?;
        let value = get_value(storage, APP_KEY)?;
        Some(value)
    }
}

// Panels
impl App {
    fn panels(&mut self, ctx: &Context) {
        self.top_panel(ctx);
        self.bottom_panel(ctx);
        self.left_panel(ctx);
        self.central_panel(ctx);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
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
                let mut behavior = Behavior::new();
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
    fn top_panel(&mut self, ctx: &Context) {
        TopBottomPanel::top("TopPanel").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    // Left panel
                    ui.toggle_value(
                        &mut self.left_panel,
                        RichText::new(SIDEBAR_SIMPLE).size(ICON_SIZE),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("left_panel"));
                    });
                    ui.separator();
                    // Light/Dark
                    ui.light_dark_button(ICON_SIZE);
                    ui.separator();
                    // Reset app
                    if ui
                        .button(RichText::new(TRASH).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("reset_application"));
                        })
                        .clicked()
                    {
                        *self = Default::default();
                    }
                    ui.separator();
                    // Reset app
                    if ui
                        .button(RichText::new(ARROWS_CLOCKWISE).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("reset_gui"));
                        })
                        .clicked()
                    {
                        // Cache
                        let caches = ui.memory_mut(|memory| memory.caches.clone());
                        // Data
                        let data = ui.memory_mut(|memory| {
                            let mut data = IdTypeMap::default();
                            // Github token
                            let id = Id::new(GITHUB_TOKEN);
                            if let Some(github_token) = memory.data.get_persisted::<String>(id) {
                                data.insert_persisted(id, github_token)
                            }
                            data
                        });
                        ui.memory_mut(|memory| {
                            memory.caches = caches;
                            memory.data = data;
                        });
                        ui.ctx().set_localizations();
                    }
                    ui.separator();
                    if ui
                        .button(RichText::new(SQUARE_SPLIT_VERTICAL).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("vertical"));
                        })
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Vertical);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(SQUARE_SPLIT_HORIZONTAL).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("horizontal"));
                        })
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Horizontal);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(GRID_FOUR).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("grid"));
                        })
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Grid);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(TABS).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("tabs"));
                        })
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Tabs);
                            }
                        }
                    }
                    ui.separator();
                    // ui.menu_button(RichText::new(GEAR).size(ICON_SIZE), |ui| {
                    //     ui.horizontal(|ui| {
                    //         ui.label(ui.localize("github token"));
                    //         let id = Id::new("GithubToken");
                    //         let mut github_token = ui.data_mut(|data| {
                    //             data.get_persisted::<String>(id).unwrap_or_default()
                    //         });
                    //         if ui.text_edit_singleline(&mut github_token).changed() {
                    //             ui.data_mut(|data| data.insert_persisted(id, github_token));
                    //         }
                    //         if ui.button(RichText::new(TRASH).heading()).clicked() {
                    //             ui.data_mut(|data| data.remove::<String>(id));
                    //         }
                    //     });
                    //     ui.horizontal(|ui| {
                    //         ui.label(ui.localize("christie"));
                    //         let pane = Pane::christie();
                    //         let tile_id = self.tree.tiles.find_pane(&pane);
                    //         let mut selected = tile_id.is_some();
                    //         if ui
                    //             .toggle_value(&mut selected, RichText::new(TABLE).heading())
                    //             .on_hover_text("Christie")
                    //             .clicked()
                    //         {
                    //             if selected {
                    //                 self.tree.insert_pane::<VERTICAL>(pane);
                    //             } else {
                    //                 self.tree.tiles.remove(tile_id.unwrap());
                    //             }
                    //         }
                    //     });
                    // });
                    if ui
                        .button(RichText::new(GEAR).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("settings"));
                        })
                        .clicked()
                    {
                        self.settings.open = !self.settings.open;
                    }
                    ui.separator();
                    // // Configuration
                    // let frames = self.data.selected();
                    // ui.add_enabled_ui(!frames.is_empty(), |ui| {
                    //     if ui
                    //         .button(RichText::new(ConfigurationPane::icon()).size(ICON_SIZE))
                    //         .on_hover_localized("configuration")
                    //         .clicked()
                    //     {
                    //         let pane = Pane::Configuration(ConfigurationPane::new(frames));
                    //         self.tree.insert_pane::<VERTICAL>(pane);
                    //     }
                    // });
                    // Create
                    if ui
                        .button(RichText::new(PLUS).size(ICON_SIZE))
                        .on_hover_localized("create")
                        .clicked()
                    {
                        let data = DataFrame::empty_with_schema(&SCHEMA);
                        let mut meta = Metadata::default();
                        meta.insert(NAME.to_owned(), "Untitled".to_owned());
                        meta.insert(DATE.to_owned(), Local::now().date_naive().to_string());
                        self.data.add(MetaDataFrame::new(meta, data));
                    }
                    // Load
                    ui.add(Presets);
                    ui.separator();
                    if ui
                        .button(RichText::new(CLOUD_ARROW_DOWN).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("load"));
                        })
                        .clicked()
                    {
                        self.github.toggle(ui);
                    }
                    ui.separator();

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // About
                        if ui
                            .button(RichText::new(INFO).size(ICON_SIZE))
                            .on_hover_localized("about")
                            .clicked()
                        {
                            self.about.open ^= true;
                        }
                        ui.separator();
                        // Locale
                        ui.locale_button().on_hover_ui(|ui| {
                            ui.label(ui.localize("language"));
                        });
                        ui.separator();
                    });
                });
            });
        });
    }
}

// Windows
impl App {
    fn windows(&mut self, ctx: &Context) {
        self.about.show(ctx);
        self.github.show(ctx);
        self.settings.show(ctx);
    }
}

// Copy/Paste, Drag&Drop
impl App {
    fn data(&mut self, ctx: &Context) {
        if let Some(frame) = ctx.data_mut(|data| data.remove_temp::<MetaDataFrame>(Id::new(DATA))) {
            self.data.add(frame);
        }
    }

    fn configure(&mut self, ctx: &Context) {
        if let Some(frames) =
            ctx.data_mut(|data| data.remove_temp::<Vec<MetaDataFrame>>(Id::new(CONFIGURE)))
        {
            self.tree
                .insert_pane::<VERTICAL>(Pane::configuration(frames));
        }
    }

    fn calculate(&mut self, ctx: &Context) {
        if let Some((frames, index)) =
            ctx.data_mut(|data| data.remove_temp::<(Vec<MetaDataFrame>, usize)>(Id::new(CALCULATE)))
        {
            self.tree
                .insert_pane::<VERTICAL>(Pane::calculation(frames, index));
        }
    }

    fn compose(&mut self, ctx: &Context) {
        if let Some((frames, index)) = ctx.data_mut(|data| {
            data.remove_temp::<(Vec<MetaDataFrame>, Option<usize>)>(Id::new(COMPOSE))
        }) {
            self.tree
                .insert_pane::<HORIZONTAL>(Pane::composition(frames, index));
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
            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
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
                let _ = self.parse(dropped_file);
            }
        }
    }

    #[instrument(skip_all, err)]
    fn parse(&mut self, dropped_file: DroppedFile) -> Result<()> {
        let bytes = dropped_file.bytes()?;
        trace!(?bytes);
        let frame = MetaDataFrame::read_parquet(Cursor::new(bytes))?;
        self.data.add(frame);
        Ok(())
    }

    // fn paste(&mut self, ctx: &Context) {
    //     if !ctx.memory(|memory| memory.focused().is_some()) {
    //         ctx.input(|input| {
    //             for event in &input.raw.events {
    //                 if let Event::Paste(paste) = event {
    //                     if let Err(error) = self.parse(paste) {
    //                         error!(?error);
    //                         self.toasts
    //                             .error(error.to_string().chars().take(64).collect::<String>())
    //                             .set_duration(Some(Duration::from_secs(5)))
    //                             .set_closable(true);
    //                     }
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
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per
    /// second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.data(ctx);
        self.configure(ctx);
        self.calculate(ctx);
        self.compose(ctx);
        // Pre update
        self.panels(ctx);
        self.windows(ctx);
        // Post update
        self.drag_and_drop(ctx);
    }
}

mod computers;
mod data;
mod identifiers;
mod panes;
mod presets;
mod text;
mod widgets;
mod windows;
