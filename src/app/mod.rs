use self::{
    data::Data,
    identifiers::{DATA, ERROR, GITHUB_TOKEN},
    panes::{Pane, behavior::Behavior, configuration::Pane as ConfigurationPane},
    windows::{About, GithubWindow},
};
use crate::localization::ContextExt as _;
use anyhow::Error;
use chrono::Local;
use eframe::{APP_KEY, CreationContext, Storage, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, Context, FontDefinitions, Frame, Id, LayerId, Layout,
    Order, RichText, ScrollArea, SidePanel, Sides, TextStyle, TopBottomPanel, Visuals, menu::bar,
    util::IdTypeMap, warn_if_debug_build,
};
use egui_ext::{DroppedFileExt as _, HoveredFileExt, LightDarkButton};
use egui_l20n::UiExt as _;
use egui_notify::Toasts;
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{
        ARROWS_CLOCKWISE, CLOUD_ARROW_DOWN, GEAR, GRID_FOUR, INFO, PLUS, SIDEBAR_SIMPLE,
        SQUARE_SPLIT_HORIZONTAL, SQUARE_SPLIT_VERTICAL, TABS, TRASH,
    },
};
use egui_tiles::{ContainerKind, Tile, Tree};
use egui_tiles_ext::{TreeExt as _, VERTICAL};
use metadata::{MetaDataFrame, Metadata};
use panes::configuration::SCHEMA;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    borrow::BorrowMut,
    fmt::Write,
    io::Cursor,
    str,
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};
use tracing::{error, info, trace};
use windows::SettingsWindow;

/// IEEE 754-2008
const MAX_PRECISION: usize = 16;
const NOTIFICATIONS_DURATION: Duration = Duration::from_secs(15);
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

    // Data channel
    #[serde(skip)]
    data_channel: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    #[serde(skip)]
    error_channel: (Sender<Error>, Receiver<Error>),

    // Windows
    #[serde(skip)]
    about: About,
    github: GithubWindow,
    settings: SettingsWindow,

    // Notifications
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for App {
    fn default() -> Self {
        Self {
            left_panel: true,
            tree: Tree::empty("CentralTree"),
            data: Default::default(),
            data_channel: channel(),
            error_channel: channel(),
            toasts: Default::default(),
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
        // cc.egui_ctx.set_localization(
        //     langid!("ru"),
        //     Localization::new(language_identifier).with_sources(RU),
        // );
        // cc.egui_ctx.set_localization(
        //     langid!("ru"),
        //     Localization::new(language_identifier).with_sources(RU),
        // );
        custom_style(&cc.egui_ctx);

        // return Default::default();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let app = Self::load(cc).unwrap_or_default();
        app.context(&cc.egui_ctx);
        app
    }

    fn load(cc: &CreationContext) -> Option<Self> {
        let storage = cc.storage?;
        let value = get_value(storage, APP_KEY)?;
        Some(value)
    }

    fn context(&self, ctx: &Context) {
        ctx.data_mut(|data| {
            // Data channel
            data.insert_temp(*DATA, self.data_channel.0.clone());
            // Error channel
            data.insert_temp(*ERROR, self.error_channel.0.clone());
        });
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
            bar(ui, |ui| {
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
                        self.context(ctx);
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
                            if let Some(github_token) =
                                memory.data.get_persisted::<String>(*GITHUB_TOKEN)
                            {
                                data.insert_persisted(*GITHUB_TOKEN, github_token)
                            }
                            data
                        });
                        ui.memory_mut(|memory| {
                            memory.caches = caches;
                            memory.data = data;
                        });
                        ui.ctx().set_localizations();
                        self.context(ctx);
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
                    // Configuration
                    let frames = self.data.checked();
                    ui.add_enabled_ui(!frames.is_empty(), |ui| {
                        if ui
                            .button(RichText::new(ConfigurationPane::icon()).size(ICON_SIZE))
                            .on_hover_ui(|ui| {
                                ui.label(ui.localize("configuration"));
                            })
                            .clicked()
                        {
                            let pane = Pane::Configuration(ConfigurationPane::new(frames));
                            self.tree.insert_pane::<VERTICAL>(pane);
                        }
                    });
                    // Create
                    if ui
                        .button(RichText::new(PLUS).size(ICON_SIZE))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("create"));
                        })
                        .clicked()
                    {
                        let data_frame = DataFrame::empty_with_schema(&SCHEMA);
                        self.data.add(MetaDataFrame {
                            meta: Metadata {
                                version: None,
                                name: "Untitled".to_owned(),
                                description: "".to_owned(),
                                authors: Vec::new(),
                                date: Some(Local::now().date_naive()),
                            },
                            data: data_frame,
                        });
                    }
                    // Load
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
                            .on_hover_ui(|ui| {
                                ui.label(ui.localize("about"));
                            })
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

// Notifications
impl App {
    fn notifications(&mut self, ctx: &Context) {
        self.toasts.show(ctx);
    }
}

// Copy/Paste, Drag&Drop
impl App {
    fn calculate(&mut self, ctx: &Context) {
        if let Some((frames, index)) = ctx
            .data_mut(|data| data.remove_temp::<(Vec<MetaDataFrame>, usize)>(Id::new("Calculate")))
        {
            self.tree
                .insert_pane::<VERTICAL>(Pane::calculation(frames, index));
        }
    }

    fn compose(&mut self, ctx: &Context) {
        if let Some((frames, index)) = ctx.data_mut(|data| {
            data.remove_temp::<(Vec<MetaDataFrame>, Option<usize>)>(Id::new("Compose"))
        }) {
            self.tree
                .insert_pane::<VERTICAL>(Pane::composition(frames, index));
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
            for dropped in dropped_files {
                trace!(?dropped);
                let bytes = match dropped.bytes() {
                    Ok(bytes) => bytes,
                    Err(error) => {
                        error!(%error);
                        self.toasts
                            .error(format!("{}: {error}", dropped.display()))
                            .closable(true)
                            .duration(Some(NOTIFICATIONS_DURATION));
                        continue;
                    }
                };
                trace!(?bytes);
                self.data_channel.0.send(bytes).ok();
            }
        }
    }

    fn parse(&mut self, ctx: &Context) {
        for bytes in self.data_channel.1.try_iter() {
            trace!(?bytes);
            match MetaDataFrame::read(Cursor::new(bytes)) {
                Ok(frame) => {
                    trace!(?frame);
                    self.data.add(frame);
                    ctx.request_repaint();
                }
                Err(error) => error!(%error),
            };
        }
    }

    fn error(&mut self, ctx: &Context) {
        let available_width = ctx.available_rect().width() / 2.0;
        for error in self.error_channel.1.try_iter() {
            self.toasts
                .error(error.to_string())
                .width(available_width)
                .duration(Some(NOTIFICATIONS_DURATION))
                .closable(true);
        }
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
        self.calculate(ctx);
        self.compose(ctx);
        // Pre update
        self.panels(ctx);
        self.windows(ctx);
        self.notifications(ctx);
        // Post update
        self.drag_and_drop(ctx);
        self.parse(ctx);
        self.error(ctx);
    }
}

/// Extension methods for [`Context`]
pub(crate) trait ContextExt {
    fn error(&self, error: impl Into<Error>);
}

impl ContextExt for Context {
    fn error(&self, error: impl Into<Error>) {
        let error = error.into();
        error!(%error);
        if let Some(sender) = self.data_mut(|data| data.get_temp::<Sender<Error>>(*ERROR)) {
            sender.send(error).ok();
        }
    }
}

/// Extension methods for [`Result`]
pub trait ResultExt<T, E> {
    fn context(self, ctx: &Context) -> Option<T>;
}

impl<T, E: Into<Error>> ResultExt<T, E> for Result<T, E> {
    fn context(self, ctx: &Context) -> Option<T> {
        self.map_err(|error| ctx.error(error)).ok()
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
