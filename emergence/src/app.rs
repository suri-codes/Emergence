use egui::{Color32, RichText};
use egui_async::StateWithData::*;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{error, info};

use egui_async::{Bind, EguiAsyncPlugin};
use egui_file_dialog::FileDialog;
use emergence_zk::{Kasten, KastenHandle, ZettelId, ZkError};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct EmergenceApp {
    // Example stuff:

    // #[serde(skip)] // This how you opt-out of serialization of a field
    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,

    kasten_bind: Bind<KastenHandle, ZkError>,
    kasten_sender: tokio::sync::mpsc::Sender<KastenHandle>,

    curr_kasten_id: Option<ZettelId>,
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

        let (tx, rx) = tokio::sync::mpsc::channel::<Arc<Mutex<Kasten>>>(15);

        // we spawn a task here who is responsible for keeping the kasten in sync with the fs
        tokio::spawn(async move { Self::kasten_watcher(rx).await });

        Self {
            file_dialog: FileDialog::new(),
            picked_file: None,
            kasten_bind: Bind::default(),
            curr_kasten_id: None,
            kasten_sender: tx,
        }
    }

    /// Runs indefinetly, will call `watch` on the latest `Kasten` sent to the receiver.
    pub async fn kasten_watcher(mut rx: Receiver<KastenHandle>) {
        let mut current_kasten_handle: Option<JoinHandle<Result<(), ZkError>>> = None;
        loop {
            if let Some(k_handle) = rx.recv().await {
                info!(
                    "received kasten: {:#?}",
                    k_handle.lock().expect("should never be poisoned").id
                );
                if let Some(old_kasten_handle) = current_kasten_handle {
                    old_kasten_handle.abort();
                }

                let k_handle = k_handle.clone();
                current_kasten_handle = Some(tokio::spawn(async move {
                    Kasten::watch(k_handle)
                        .await
                        .inspect_err(|e| error!("WATCHER SHUTDOWN!! {e:#?}"))
                }));
            }
        }
    }
}

impl eframe::App for EmergenceApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // this is for the kasten bind thingy
        ctx.plugin_or_default::<EguiAsyncPlugin>();

        egui::SidePanel::left("left_panel")
            .resizable(true)
            //NOTE: these are some bullshit values lol
            .default_width(1500.0)
            .width_range(80.0..=1000.0)
            .show(ctx, |ui| {
                match self.kasten_bind.state() {
                    Finished(k_handle) => {
                        let k = k_handle.lock().expect("must not be poisoned");
                        if let Some(recently_edited) = k.most_recently_edited {
                            let zettel =
                                k.graph.node(recently_edited).expect("must exist").payload();

                            ui.vertical_centered(|ui| {
                                ui.heading(zettel.front_matter.title.clone());
                            });
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                let mut cache = CommonMarkCache::default();
                                CommonMarkViewer::new().show(ui, &mut cache, &zettel.content);
                            });
                        } else {
                            egui::ScrollArea::vertical().show(ui, |_| {});
                        };
                    }
                    _ => {
                        ui.spinner();
                    }
                };
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            self.file_dialog.update(ctx);

            // Check if the user picked a file.
            if let Some(path) = self.file_dialog.take_picked() {
                self.picked_file = Some(path.to_path_buf());
                self.kasten_bind.clear();
                self.kasten_bind
                    .request(async { Kasten::parse(path).await.map(|k| Arc::new(Mutex::new(k))) });
            }

            match self.kasten_bind.state() {
                Idle => {
                    if ui
                        .button(RichText::new("Open ZettelKasten").size(24.0))
                        .clicked()
                    {
                        self.file_dialog.pick_directory();
                    };
                }
                Pending => {
                    ui.spinner();
                }
                Finished(k_handle) => {
                    let mut kg = k_handle.lock().expect("should never be poisoned");
                    match self.curr_kasten_id {
                        // if the current id == the kasten we are binded to, we do nothing
                        Some(ref id) if *id == kg.id => {}
                        // other wise we send an update
                        _ => {
                            let sender = self.kasten_sender.clone();
                            let k_clone = k_handle.clone();
                            info!("sending kasten to watcher thread: {:#?}", kg.id);
                            tokio::spawn(async move {
                                sender.send(k_clone).await.expect("lol");
                            });
                            self.curr_kasten_id = Some(kg.id.clone());
                        }
                    }

                    ui.horizontal_top(|ui| {
                        ui.heading(kg.name.clone());

                        if ui
                            .button(RichText::new("⏷").size(14.0))
                            .on_hover_text("Select Different ZettelKasten")
                            .clicked()
                        {
                            self.file_dialog.pick_directory();
                        };
                    });

                    let g = &mut kg.graph;

                    type L = egui_graphs::LayoutForceDirected<
                        egui_graphs::FruchtermanReingoldWithCenterGravity,
                    >;
                    type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;
                    let mut view = egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::new(g);
                    ui.add(&mut view);
                }
                Failed(err) => {
                    // show error message
                    ui.colored_label(
                        Color32::RED,
                        format!("Error opening ZettelKasten! :{:#?}", err),
                    );

                    if ui
                        .button(RichText::new("Open ZettelKasten").size(24.0))
                        .clicked()
                    {
                        self.file_dialog.pick_directory();
                    };
                }
            }

            // ui.separator();

            // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     powered_by_egui_and_eframe(ui);
            //     egui::warn_if_debug_build(ui);
            // });
            // ui.separator()
        });
    }
}

#[expect(unused)]
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
