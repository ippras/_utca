// #![feature(hash_set_entry)]
// #![feature(debug_closure_helpers)]
#![feature(box_patterns)]
#![feature(result_option_map_or_default)]

pub use app::App;

mod app;
mod r#const;
mod export;
mod localization;
mod presets;
mod special;

// mod properties;
// mod widgets;

mod utils;
