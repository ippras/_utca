#![feature(array_try_map)]
#![feature(decl_macro)]
#![feature(if_let_guard)]
#![feature(result_option_map_or_default)]

pub use app::App;

mod app;
mod assets;
mod r#const;
mod export;
mod localization;
mod macros;
mod text;
mod utils;
