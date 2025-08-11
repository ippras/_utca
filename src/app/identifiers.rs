use egui::Id;
use std::sync::LazyLock;

pub(crate) static DATA: LazyLock<Id> = LazyLock::new(|| Id::new("Data"));
// pub(crate) static ERROR: LazyLock<Id> = LazyLock::new(|| Id::new("Error"));
pub(crate) static GITHUB_TOKEN: LazyLock<Id> = LazyLock::new(|| Id::new("GithubToken"));
