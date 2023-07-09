use std::collections::HashMap;

use crossbeam::channel::{unbounded, Receiver};
use egui::{
    text::LayoutJob, Align, Button, Color32, Context, CursorIcon, FontFamily, FontId, InputState,
    Label, Sense, Stroke, Style, TextEdit, TextFormat, TextStyle, Ui, WidgetText,
};
use egui_graphs::{add_edge, add_node, Graph, Node, SettingsNavigation, SettingsStyle};
use egui_graphs::{add_node_custom, SettingsInteraction};
use log::error;
use log::info;
use petgraph::{
    stable_graph::{NodeIndex, StableGraph},
    Directed,
};
use rand::seq::IteratorRandom;
use rand::Rng;
use reqwest::Error;
use tokio::task::JoinHandle;

use crate::{
    node,
    state::{next, Fork, State},
    url::{self, Url},
    url_retriever::UrlRetriever,
};

const HEADING: &str = "Wiki Links";
const DESCRIPTION : &str = "A demo application for egui_graphs widget. This application will display a graph of a wikipedia article links.";
const TOOLTIP: &str = "enter a Wikipedia article url and hit enter";
const ERROR_MSG: &str = "enter a valid wikipedia article url";

const COLOR_ACCENT: Color32 = Color32::from_rgb(128, 128, 255);
const COLOR_SUB_ACCENT: Color32 = Color32::from_rgb(64, 64, 128);
const COLOR_ERROR: Color32 = Color32::from_rgb(255, 64, 64);

const CURSOR_WIDTH: f32 = 5.;

const EDGE_WEIGHT: f32 = 0.05;

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

    pub fn update(&mut self, ctx: &Context, ui: &mut Ui) {
        self.size_section = ui.available_height() / 5.;
        self.size_margin = ui.available_height() / 20.;

        ui.set_style(self.style.clone());

        self.handle_state();
        self.draw(ui);
        self.handle_keys(ctx);
    }

    fn handle_state(&mut self) {
        match self.state {
            State::GraphAndLoading => self.handle_state_graph_and_loading(),
            State::GraphAndLoadingError | State::Input | State::InputError | State::Graph => (),
        }
    }

    fn draw(&mut self, ui: &mut Ui) {
        match self.state {
            State::Input => self.draw_input(ui),
            State::InputError => self.draw_input_error(ui),
            State::GraphAndLoading => self.draw_graph_and_loading(ui),
            State::Graph => self.draw_graph(ui),
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

    fn draw_graph_and_loading(&mut self, ui: &mut Ui) {
        let mut w = egui_graphs::GraphView::new(&mut self.g);
        w = w.with_styles(&SettingsStyle::default().with_edge_radius_weight(EDGE_WEIGHT));
        ui.add(&mut w);
    }

    fn draw_graph(&mut self, ui: &mut Ui) {
        let mut w = egui_graphs::GraphView::new(&mut self.g);
        w = w.with_interactions(
            &SettingsInteraction::default()
                .with_selection_enabled(true)
                .with_dragging_enabled(true)
                .with_selection_depth(1),
        );
        w = w.with_navigations(
            &SettingsNavigation::default()
                .with_fit_to_screen_enabled(false)
                .with_zoom_and_pan_enabled(true),
        );
        w = w.with_styles(&SettingsStyle::default().with_edge_radius_weight(EDGE_WEIGHT));
        ui.add(&mut w);
    }

    fn draw_input_error(&mut self, ui: &mut Ui) {
        self.draw_view_input(ui, false);
    }

    fn draw_input(&mut self, ui: &mut Ui) {
        self.draw_view_input(ui, true);
    }

    fn draw_view_input(&mut self, ui: &mut Ui, url_valid: bool) {
        ui.vertical_centered(|ui| {
            ui.add_space(self.size_section);
            ui.label(header_accent(HEADING));

            ui.add_space(self.size_margin);
            ui.label(DESCRIPTION);

            ui.add_space(self.size_section);
            ui.label(TOOLTIP);

            ui.add_space(self.size_margin);
            let mut input = TextEdit::singleline(&mut self.root_article_url)
                .frame(false)
                .desired_rows(1)
                .vertical_align(Align::Center)
                .font(FontId::new(24., FontFamily::Monospace))
                .horizontal_align(Align::Center)
                .desired_width(f32::INFINITY);

            if !url_valid {
                input = input.text_color(COLOR_ERROR);
            }

            let input_response = input.show(ui).response;
            input_response.request_focus();

            if input_response.changed() {
                self.state = State::Input;
            }

            if !url_valid {
                ui.add_space(self.size_margin / 4.);
                ui.label(ERROR_MSG);
            }
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

fn header_accent(text: &str) -> impl Into<WidgetText> {
    let mut job = LayoutJob::default();
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(24., FontFamily::Monospace),
            color: COLOR_ACCENT,
            ..Default::default()
        },
    );
    WidgetText::from(job)
}
