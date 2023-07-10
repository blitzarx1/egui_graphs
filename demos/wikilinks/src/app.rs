use std::collections::HashMap;

use crossbeam::channel::{unbounded, Receiver};
use egui::{
    text::LayoutJob, Align, Button, Color32, Context, CursorIcon, FontFamily, FontId, InputState,
    Label, Sense, Stroke, Style, TextEdit, TextFormat, TextStyle, Ui, WidgetText,
};
use egui::{Area, CentralPanel, Response, ScrollArea, SidePanel};
use egui_graphs::{add_edge, add_node, Graph, Node, SettingsNavigation, SettingsStyle};
use egui_graphs::{add_node_custom, SettingsInteraction};
use log::error;
use log::info;
use petgraph::EdgeType;
use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    Directed,
};
use rand::seq::IteratorRandom;
use rand::Rng;
use reqwest::Error;
use tokio::task::JoinHandle;

use crate::views::graph::draw_view_graph;
use crate::views::input::draw_view_input;
use crate::views::style::{header_accent, COLOR_ACCENT, COLOR_SUB_ACCENT, CURSOR_WIDTH};
use crate::views::toolbox::draw_view_toolbox;
use crate::{
    node,
    state::{next, Fork, State},
    url::{self, Url},
    url_retriever::UrlRetriever,
};

#[derive(Default)]
pub struct App {
    root_article_url: String,
    state: State,

    size_section: f32,
    size_margin: f32,
    style: Style,

    active_tasks: HashMap<NodeIndex, (Receiver<Result<Url, Error>>, JoinHandle<()>)>,

    g: Graph<node::Node, (), Directed>,
}

impl App {
    pub fn new() -> Self {
        let mut style = Style::default();
        style.visuals.text_cursor_width = CURSOR_WIDTH;
        style.visuals.selection.stroke = Stroke::new(1., COLOR_ACCENT);
        style.visuals.selection.bg_fill = COLOR_SUB_ACCENT;

        App {
            style,
            ..Default::default()
        }
    }

    pub fn update(&mut self, ctx: &Context) {
        ctx.set_style(self.style.clone());

        self.handle_state();
        self.draw(ctx);
        self.handle_keys(ctx);
    }

    fn handle_state(&mut self) {
        match self.state {
            State::GraphAndLoading => self.handle_state_graph_and_loading(),
            State::GraphAndLoadingError | State::Input | State::InputError | State::Graph => (),
        }
    }

    fn draw(&mut self, ctx: &Context) {
        match self.state {
            State::Input => self.draw_input(ctx),
            State::InputError => self.draw_input_error(ctx),
            State::GraphAndLoading => self.draw_graph_and_loading(ctx),
            State::Graph => self.draw_graph(ctx),
            State::GraphAndLoadingError => todo!(),
        }
    }

    /// Checks for results from the url retriever for every active task. If any task is finished,
    /// moves to the next state.
    fn handle_state_graph_and_loading(&mut self) {
        match self.check_active_tasks() {
            Ok(_) => {
                if self.active_tasks.is_empty() {
                    info!("all tasks finished");
                    self.state = next(&self.state, Fork::Success);
                }
            }
            Err(err) => {
                error!("error while checking active tasks: {}", err);
                self.state = next(&self.state, Fork::Failure);
            }
        }
    }

    /// Checks for results from the url retriever for every active task.
    ///
    /// Updates the graph with the retrieved urls.
    ///
    /// If any task is finished, removes it from the active tasks.
    ///
    /// If we got any url, function returns true, otherwise false. If an error was got function returns error.
    fn check_active_tasks(&mut self) -> Result<(), Error> {
        let mut finished_tasks = Vec::new();
        self.active_tasks
            .iter()
            .for_each(
                |(parent, (receiver, join_handle))| match receiver.try_recv() {
                    Ok(result) => match result {
                        Ok(url) => {
                            info!("got new url from the retriver: {}", url.val());

                            let mut rng = rand::thread_rng();
                            let random_n_loc =
                                self.g.node_weights().choose(&mut rng).unwrap().location();

                            let idx =
                                add_node_custom(&mut self.g, &node::Node::new(url), |_, n| {
                                    let mut rng = rand::thread_rng();
                                    Node::new(
                                        egui::Vec2 {
                                            x: random_n_loc.x + rng.gen_range(-100.0..100.),
                                            y: random_n_loc.y + rng.gen_range(-100.0..100.),
                                        },
                                        n.clone(),
                                    )
                                    .with_label(n.url().val().to_string())
                                    .with_color(
                                        match n.url().is_wiki_article() {
                                            true => COLOR_ACCENT,
                                            false => Color32::GRAY,
                                        },
                                    )
                                });
                            add_edge(&mut self.g, *parent, idx, &());
                        }
                        Err(err) => {
                            error!("got error from the retriver: {}", err);
                        }
                    },

                    Err(_) => {
                        if join_handle.is_finished() {
                            finished_tasks.push(*parent);
                        }
                    }
                },
            );

        finished_tasks.iter().for_each(|finished| {
            info!(
                "task finished; received all children urls for: {}",
                self.g
                    .node_weight(*finished)
                    .unwrap()
                    .data()
                    .unwrap()
                    .url()
                    .val()
            );
            self.active_tasks.remove(finished);
        });

        Ok(())
    }

    fn handle_keys(&mut self, ctx: &Context) {
        ctx.input(|i| match self.state {
            State::Input => self.handle_keys_input(i),
            State::InputError
            | State::GraphAndLoading
            | State::GraphAndLoadingError
            | State::Graph => (),
        });
    }

    fn draw_input_error(&mut self, ctx: &Context) {
        let input_resp = CentralPanel::default().show(ctx, |ui| {
            draw_view_input(
                &mut self.root_article_url,
                ui,
                false,
                ui.available_height() / 5.,
                ui.available_height() / 20.,
            )
        });

        if input_resp.inner.changed() {
            self.state = next(&self.state, Fork::Success);
        }
    }

    fn draw_input(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            draw_view_input(
                &mut self.root_article_url,
                ui,
                true,
                ui.available_height() / 5.,
                ui.available_height() / 20.,
            );
        });
    }

    fn draw_graph_and_loading(&mut self, ctx: &Context) {
        SidePanel::right("toolbox").resizable(true).show(ctx, |ui| {
            ui.centered_and_justified(|ui| draw_view_toolbox(ui, true));
        });
        CentralPanel::default().show(ctx, |ui| {
            draw_view_graph(&mut self.g, ui, true);
        });
    }

    fn draw_graph(&mut self, ctx: &Context) {
        SidePanel::right("toolbox")
            .resizable(true)
            .show(ctx, |ui| draw_view_toolbox(ui, false));
        CentralPanel::default().show(ctx, |ui| {
            draw_view_graph(&mut self.g, ui, false);
        });
    }

    fn handle_keys_input(&mut self, i: &InputState) {
        if i.key_pressed(egui::Key::Enter) {
            match url::Url::new(&self.root_article_url) {
                Ok(u) => {
                    if !u.is_wiki() {
                        self.state = next(&self.state, Fork::Failure);
                        return;
                    }

                    self.g = StableGraph::new();

                    let idx = add_node_custom(&mut self.g, &node::Node::new(u.clone()), |_, n| {
                        let mut rng = rand::thread_rng();
                        Node::new(
                            egui::Vec2 {
                                x: rng.gen_range(-100.0..100.),
                                y: rng.gen_range(-100.0..100.),
                            },
                            n.clone(),
                        )
                        .with_label(n.url().val().to_string())
                        .with_color(COLOR_ACCENT)
                    });

                    let (sender, receiver) = unbounded();
                    let retriever = UrlRetriever::new(sender);

                    info!("started retriever for {}", u.val());

                    self.active_tasks.insert(idx, (receiver, retriever.run(u)));
                    self.state = next(&self.state, Fork::Success);
                }
                Err(_) => {
                    self.state = next(&self.state, Fork::Failure);
                }
            };
        };
    }
}
