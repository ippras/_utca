// #![feature(hash_set_entry)]
// #![feature(debug_closure_helpers)]
#![feature(box_patterns)]
#![feature(debug_closure_helpers)]
#![feature(if_let_guard)]
#![feature(result_option_map_or_default)]

pub use app::App;

mod app;
mod r#const;
mod export;
mod localization;
mod presets;
mod special;
mod text;

// mod properties;
// mod widgets;

mod utils;
