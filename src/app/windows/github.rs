use crate::{
    app::identifiers::{DATA, GITHUB_TOKEN as ID},
    utils::spawn,
};
use anyhow::{Error, Result};
use base64::prelude::*;
use egui::{
    CollapsingHeader, Context, Grid, Id, Label, RichText, ScrollArea, Sense, TextEdit, Ui, Widget,
    Window,
};
use egui_phosphor::regular::CLOUD_ARROW_DOWN;
use ehttp::{Headers, Request, Response, fetch, fetch_async};
use itertools::{Either, Itertools};
use poll_promise::Promise;
use radix_trie::{SubTrie, Trie, TrieCommon, iter::Children};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    env::var,
    f32::INFINITY,
    fmt::{Debug, Display},
    future::Future,
    path::{Components, Path},
    sync::mpsc::Sender,
};
use tracing::{error, info, trace, warn};
use url::Url;

// https://api.github.com/repos/ippras/utca/gh-pages/configs/H242_Tamia_Peroxide.toml
// /repos/repos/ippras/git/trees/{tree_sha}
// const URL: &str = "https://api.github.com/repos/ippras/utca/contents/configs";
// const URL: &str = "https://api.github.com/repos/ippras/utca/contents/configs";
// /repos/{owner}/{repo}/git/trees/{tree_sha}
// https://api.github.com/repos/ippras/utca/git/trees/gh-pages?recursive=true
// https://api.github.com/repos/ippras/utca/git/trees/gh-pages/configs?recursive=true

const ENVIRONMENT: Option<&str> = option_env!("GITHUB_TOKEN");
const URL: &str = "https://api.github.com/repos/ippras/utca-configs/git/trees/main?recursive=true";

/// Github window
///
/// `github.com tree` renders a nested list of debugger values.
#[derive(Deserialize, Serialize)]
pub struct GithubWindow {
    pub open: bool,
    #[serde(skip, default = "promise_none")]
    promise: Promise<Option<Tree>>,
}

impl GithubWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            promise: Promise::from_ready(None),
        }
    }

    pub fn toggle(&mut self, ui: &Ui) {
        self.open ^= true;
        self.promise = if self.open {
            let mut github_token = ui.data_mut(|data| data.get_persisted::<String>(*ID));
            github_token = github_token.or(ENVIRONMENT.map(ToOwned::to_owned));
            let Some(github_token) = github_token else {
                warn!("GITHUB_TOKEN not found");
                return;
            };
            load_tree(URL, &github_token)
        } else {
            Promise::from_ready(None)
        };
    }

    // if self.show_confirmation_dialog {
    //     egui::Window::new("Do you want to quit?")
    //         .collapsible(false)
    //         .resizable(false)
    //         .show(ctx, |ui| {
    //             ui.horizontal(|ui| {
    //                 if ui.button("No").clicked() {
    //                     self.show_confirmation_dialog = false;
    //                     self.allowed_to_close = false;
    //                 }
    //                 if ui.button("Yes").clicked() {
    //                     self.show_confirmation_dialog = false;
    //                     self.allowed_to_close = true;
    //                     ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    //                 }
    //             });
    //         });
    // }

    pub fn show(&mut self, ctx: &Context) {
        Window::new(format!("{CLOUD_ARROW_DOWN} Load config"))
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.visuals_mut().collapsing_header_frame = true;
                // ui.collapsing(localize!("settings"), |ui| {
                //     Grid::new(ui.next_auto_id()).show(ui, |ui| {
                //         ui.label(localize!("github token"));
                //         // ui.text_edit_singleline(&mut self.token)
                //         ui.add(TextEdit::singleline(&mut self.token).desired_width(INFINITY));
                //         ctx.request_repaint();
                //     });
                // });
                // ui.separator();
                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(Some(tree)) = self.promise.ready() {
                        let mut trie = Trie::new();
                        for node in &tree.tree {
                            // if node.r#type == "blob" {
                            // if let Some(path) = node.path.strip_prefix("configs/") {
                            //     trie.insert(path, &*node.url);
                            // }
                            trie.insert(&*node.path, &*node.url);
                        }
                        show_children(ui, trie.children());
                    } else {
                        ui.spinner();
                    }
                });
            });
    }
}

impl Default for GithubWindow {
    fn default() -> Self {
        Self::new()
    }
}

fn promise_none() -> Promise<Option<Tree>> {
    Promise::from_ready(None)
}

fn show_children(ui: &mut Ui, children: Children<'_, &str, &str>) {
    for trie in children.sorted_by_cached_key(|trie| trie.is_leaf()) {
        if let Some(&path) = trie.key() {
            let name = path.rsplit_once('/').map_or(path, |(_, suffix)| suffix);
            if trie.is_leaf() {
                if let Some(&url) = trie.value() {
                    ui.horizontal(|ui| {
                        if ui.button(CLOUD_ARROW_DOWN).on_hover_text(url).clicked() {
                            load_blob(ui.ctx(), url);
                        }
                        ui.label(name);
                    });
                }
            } else {
                ui.collapsing(RichText::new(name).heading(), |ui| {
                    show_children(ui, trie.children());
                });
            }
        } else {
            show_children(ui, trie.children());
        }
    }
}

fn load_tree(url: impl ToString, github_token: impl ToString) -> Promise<Option<Tree>> {
    let url = url.to_string();
    let github_token = github_token.to_string();
    spawn(async {
        match try_load_tree(url, github_token).await {
            Ok(tree) => Some(tree),
            Err(error) => {
                error!(%error);
                None
            }
        }
    })
    // let (sender, promise) = Promise::new();
    // fetch(request, move |response| {
    //     if let Err(error) = try_load_tree(sender, response) {
    //         error!(%error);
    //     }
    //     // match response {
    //     // Ok(response) => match response.json::<Tree>() {
    //     //     Ok(tree) => sender.send(tree),
    //     //     Err(error) => {
    //     //         error!(%error);
    //     //         info!("Status code: {}", response.status);
    //     //         sender.send(Default::default());
    //     //     }
    //     // },
    //     // Err(error) => {
    //     //     error!(%error);
    //     //     sender.send(Default::default());
    //     // }
    // });
    // promise
}

fn load_blob(ctx: &Context, url: impl ToString) {
    let ctx = ctx.clone();
    let url = url.to_string();
    let _ = spawn(async move {
        match try_load_blob(url).await {
            Ok(blob) => ctx.data_mut(|data| {
                if let Some(sender) = data.get_temp::<Sender<Vec<u8>>>(*DATA) {
                    sender.send(blob).ok();
                }
            }),
            Err(error) => error!(%error),
        }
    });
}

async fn try_load_tree(url: impl ToString, github_token: impl Display) -> Result<Tree> {
    let request = Request {
        headers: Headers::new(&[
            ("Accept", "application/vnd.github+json"),
            ("Authorization", &format!("Bearer {github_token}")),
            ("X-GitHub-Api-Version", "2022-11-28"),
        ]),
        ..Request::get(url)
    };
    let response = fetch_async(request).await.map_err(Error::msg)?;
    let tree = response.json::<Tree>()?;
    Ok(tree)
}

async fn try_load_blob(url: impl ToString) -> Result<Vec<u8>> {
    let request = Request::get(url);
    let response = fetch_async(request).await.map_err(Error::msg)?;
    let blob = response.json::<Blob>()?;
    trace!(?blob);
    let mut bytes = Vec::new();
    for line in blob.content.split_terminator('\n') {
        bytes.append(&mut BASE64_STANDARD.decode(line)?);
    }
    Ok(bytes)
}

/// Tree
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct Tree {
    sha: String,
    url: String,
    truncated: bool,
    tree: Vec<Node>,
}

/// Node
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Node {
    path: String,
    mode: String,
    r#type: String,
    sha: String,
    size: Option<u64>,
    url: String,
}

/// Blob
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Blob {
    content: String,
    encoding: String,
    url: String,
    sha: String,
    size: u64,
    node_id: String,
}

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct Entry {
//     pub name: String,
//     pub size: usize,
//     pub download_url: Option<Url>,
//     pub r#type: Type,
// }

// #[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
// #[serde(rename_all = "lowercase")]
// pub enum Type {
//     Dir,
//     File,
// }
