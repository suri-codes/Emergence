use egui::Color32;
use egui_graphs::Graph;
use emergence_zk::{Kasten, Link, Zettel};
use log::error;
use petgraph::{Undirected, graph::NodeIndex};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EmergenceApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    graph: EmerGraph,
}

type EmerGraph = Graph<Zettel, Link, Undirected>;

impl Default for EmergenceApp {
    fn default() -> Self {
        let kasten = Kasten::parse("./test_kasten")
            .inspect_err(|e| error!("{e:#?}"))
            .expect("test_kasten missing, try generating it");

        let mut graph = EmerGraph::from(&kasten.graph);

        let node_ids: Vec<_> = graph
            .nodes_iter()
            .map(|(idx, _)| idx)
            .collect::<Vec<NodeIndex>>();

        for node_idx in node_ids {
            let node = graph.node_mut(node_idx).expect("must exist");
            let zettel = &node.props().payload;

            node.set_label(zettel.front_matter.name.clone());
            // this should be soemthing related to the thing
            node.set_color(Color32::GREEN);
        }

        Self {
            // Example stuff:
            label: "Hello orld!".to_owned(),
            value: 2.7,
            graph,
        }
    }
}

impl EmergenceApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Default::default()
        // }
        //
        Default::default()
    }
}

impl eframe::App for EmergenceApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            // this the really crazy graph view
            // type L = LayoutForceDirected<FruchtermanReingold>;
            // type S = FruchtermanReingoldState;
            // let mut graph_view =
            //     egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::new(&mut self.graph);
            let mut graph_view = egui_graphs::GraphView::<_, _, Undirected>::new(&mut self.graph);

            ui.add(&mut graph_view);

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
