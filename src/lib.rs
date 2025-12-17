#![feature(array_try_map)]
#![feature(box_patterns)]
#![feature(debug_closure_helpers)]
#![feature(decl_macro)]
#![feature(extend_one)]
#![feature(if_let_guard)]
#![feature(result_option_map_or_default)]
#![feature(str_as_str)]

pub use app::App;

mod app;
mod assets;
mod r#const;
mod export;
mod localization;
mod macros;
mod text;
mod utils;
