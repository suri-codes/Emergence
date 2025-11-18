use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender, SyncSender},
    },
    thread::{self, JoinHandle},
};

use egui_async::{Bind, EguiAsyncPlugin};
use egui_file_dialog::FileDialog;
use emergence_zk::{Kasten, Zettel, ZettelId, ZkError, ZkResult};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::channel;
use tracing::{error, info};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EmergenceApp {
    // Example stuff:

    // #[serde(skip)] // This how you opt-out of serialization of a field
    file_dialog: FileDialog,
    picked_file: Option<PathBuf>,

    // #[serde(skip)] // This how you opt-out of serialization of a field
    // graph: EmerGraph,
    kasten_bind: Bind<Arc<Mutex<Kasten>>, ZkError>,

    kasten_sender: tokio::sync::mpsc::Sender<Arc<Mutex<Kasten>>>,

    // kasten_watcher_handle: std::thread::JoinHandle<()>,
    kasten_watcher_handle: tokio::task::JoinHandle<()>,

    kasten_sender_bind: Bind<(), ZkError>,

    curr_kasten_id: Option<ZettelId>,

    kasten_watcher_bind: Bind<(), ZkError>, // kasten: Kasten,
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

        // ok we will just block on this cuz we are dick
        // let kasten = tokio::runtime::Handle::current().block_on(async {
        //     Kasten::parse("./test_kasten")
        //         .await
        //         .inspect_err(|e| error!("{e:#?}"))
        //         .expect("test_kasten missing, try generating it")
        // });

        // Overhead: ~negligible
        // let (tx, mut rx) = tokio::sync::oneshot::channel();

        // tokio::spawn(async move {
        //     let kasten = Kasten::parse("./ZettelKasten").await.unwrap();
        //     tx.send(kasten).unwrap();
        // });

        // let kasten = loop {
        //     match rx.try_recv() {
        //         Ok(k) => break k,

        //         _ => continue,
        //     };
        // };

        // let mut x = kasten.clone();
        // tokio::spawn(async move {
        //     let res = x.watch().await;
        //     error!("{res:#?}")
        // });
        //

        let (tx, mut rx) = tokio::sync::mpsc::channel::<Arc<Mutex<Kasten>>>(15);

        use notify::{Config, Event, RecommendedWatcher};
        use tokio::sync::mpsc::{Receiver, channel};

        let kasten_watcher: tokio::task::JoinHandle<()> = tokio::spawn(async move {
            let mut current_kasten_handle: Option<tokio::task::JoinHandle<Result<(), ZkError>>> =
                None;

            loop {
                if let Some(kasten) = rx.recv().await {
                    let k = kasten.lock().expect("should never be poisoned");
                    info!("received kasten: {:#?}", k.id);
                    drop(k);
                    if let Some(old_kasten_handle) = current_kasten_handle {
                        old_kasten_handle.abort();
                    }

                    info!("we are here");

                    let k = kasten.clone();
                    current_kasten_handle = Some(tokio::spawn(async move {
                        info!("the handle was atleast spawned xd");

                        let ws = k.lock().expect("lol").ws.clone();

                        // Use recommended_watcher() to automatically select the best implementation
                        // for your platform. The `EventHandler` passed to this constructor can be a
                        // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
                        // another type the trait is implemented for.
                        //
                        // use tokio::sync::mpsc::{channel, Receiver};
                        use notify::{Config, Event, RecommendedWatcher};

                        let (tx, mut rx) = channel(1);

                        let mut watcher = RecommendedWatcher::new(
                            move |res| tx.blocking_send(res).expect("failed to send event"),
                            Config::default(),
                        )?;

                        // let (tx, mut rx) = mpsc::channel(100);
                        //    let mut watcher =
                        //        RecommendedWatcher::new(move |result: std::result::Result<Event, Error>| {
                        //            tx.blocking_send(result).expect("Failed to send event");
                        //        })?;

                        //    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

                        //    // This is a simple loop, but you may want to use more complex logic here,
                        //    // for example to handle I/O.
                        //    while let Some(res) = rx.recv().await {
                        //        tokio::spawn(async move {println!("got = {:?}", res);});
                        //    }

                        //
                        // let (mut watcher, mut rx) = async_watcher()?;

                        // Add a path to be watched. All files and directories at that path and
                        // below will be monitored for changes.
                        watcher
                            .watch(Path::new(&ws.root), RecursiveMode::Recursive)
                            .expect("lol");
                        // Block forever, printing out events as they come in

                        while let Some(res) = rx.recv().await {
                            match res {
                                Ok(event) => {
                                    info!("event: {:#?}", event);
                                    if let notify::EventKind::Modify(
                                        notify::event::ModifyKind::Data(_),
                                    ) = event.kind
                                    {
                                        for path in event.paths {
                                            info!("we are goin through shit");
                                            let Ok(z) = Zettel::from_path(&path, &ws).await.inspect_err(|e| {
                                error!("Unable to parse zettel from path: {path:#?}, error: {e:#?}")
                            }) else {
                                continue;
                            };

                                            info!("zettel: {z:#?}");

                                            let mut kasten_guard =
                                                k.lock().expect("should have worked");

                                            // actually this has the very real possibility of changing :grin:
                                            let gid = {
                                                match kasten_guard.zid_to_gid.get(&z.id) {
                                                    Some(gid) => *gid,
                                                    None => {
                                                        // this zettel was created while we have watch open, lets just add
                                                        // it to kasten_guard.thegraph and the hashmap
                                                        let gid = kasten_guard
                                                            .graph
                                                            .add_node_custom(z.clone(), |node| {
                                                                z.apply_node_transform(node)
                                                            });

                                                        kasten_guard
                                                            .zid_to_gid
                                                            .insert(z.id.clone(), gid);
                                                        gid
                                                    }
                                                }
                                            };

                                            let x = kasten_guard
                                                .graph
                                                .g_mut()
                                                .node_weight_mut(gid)
                                                .expect("must exist");
                                            (*x.payload_mut()) = z.clone();
                                            z.apply_node_transform(x);

                                            let curr_edgs = kasten_guard
                                                .graph
                                                .g()
                                                .edges(gid)
                                                .map(|e| e.weight().id())
                                                .collect::<Vec<_>>();

                                            for edge in curr_edgs {
                                                let _ = kasten_guard.graph.remove_edge(edge);
                                            }

                                            for link in z.links {
                                                let dest = *kasten_guard
                                                    .zid_to_gid
                                                    .get(&link.dest)
                                                    .expect("must exist");
                                                kasten_guard.graph.add_edge(gid, dest, link);
                                            }

                                            info!("kasten_guard.graph: {:#?} ", kasten_guard.graph);
                                        }
                                    }
                                }
                                Err(e) => error!("watch error: {:#?}", e),
                            }
                        }

                        Ok(())
                    }));
                }
            }
        });

        // let kasten_watcher: JoinHandle<()> = thread::spawn(|| {
        //     tokio::runtime::Builder::new_multi_thread()
        //         .enable_all()
        //         .build().expect("failed to build rt").block_on(

        //     async move {
        //         let mut current_kasten_handle: Option<
        //             tokio::task::JoinHandle<Result<(), ZkError>>,
        //         > = None;

        //         loop {
        //             if let Ok(kasten) = rx.recv() {
        //                 let k = kasten.lock().expect("should never be poisoned");
        //                 info!("received kasten: {:#?}", k.id);
        //                 drop(k);
        //                 if let Some(old_kasten_handle) = current_kasten_handle {
        //                     old_kasten_handle.abort();
        //                 }

        //                 info!("we are here");

        //                 let k = kasten.clone();
        //                 current_kasten_handle = Some(tokio::spawn(async move {
        //                     info!("the handle was atleast spawned xd");

        //                     let ws = k.lock().expect("lol").ws.clone();

        //                     let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

        //                     // Use recommended_watcher() to automatically select the best implementation
        //                     // for your platform. The `EventHandler` passed to this constructor can be a
        //                     // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
        //                     // another type the trait is implemented for.
        //                     let mut watcher = notify::recommended_watcher(tx)?;

        //                     // Add a path to be watched. All files and directories at that path and
        //                     // below will be monitored for changes.
        //                     watcher
        //                         .watch(Path::new(&ws.root), RecursiveMode::Recursive)
        //                         .expect("lol");
        //                     // Block forever, printing out events as they come in

        //                     while let Ok(res) = rx.recv() {
        //                         match res {
        //                             Ok(event) => {
        //                                 info!("event: {:#?}", event);
        //                                 if let notify::EventKind::Modify(
        //                                     notify::event::ModifyKind::Data(_),
        //                                 ) = event.kind
        //                                 {
        //                                     for path in event.paths {
        //                                         info!("we are goin through shit");
        //                                         let Ok(z) = Zettel::from_path(&path, &ws).await.inspect_err(|e| {
        //                         error!("Unable to parse zettel from path: {path:#?}, error: {e:#?}")
        //                     }) else {
        //                         continue;
        //                     };

        //                                         info!("zettel: {z:#?}");

        //                                         let mut kasten_guard =
        //                                             k.lock().expect("should have worked");

        //                                         // actually this has the very real possibility of changing :grin:
        //                                         let gid = {
        //                                             match kasten_guard.zid_to_gid.get(&z.id) {
        //                                                 Some(gid) => *gid,
        //                                                 None => {
        //                                                     // this zettel was created while we have watch open, lets just add
        //                                                     // it to kasten_guard.thegraph and the hashmap
        //                                                     let gid =
        //                                                         kasten_guard.graph.add_node_custom(
        //                                                             z.clone(),
        //                                                             |node| {
        //                                                                 z.apply_node_transform(node)
        //                                                             },
        //                                                         );

        //                                                     kasten_guard
        //                                                         .zid_to_gid
        //                                                         .insert(z.id.clone(), gid);
        //                                                     gid
        //                                                 }
        //                                             }
        //                                         };

        //                                         let x = kasten_guard
        //                                             .graph
        //                                             .g_mut()
        //                                             .node_weight_mut(gid)
        //                                             .expect("must exist");
        //                                         (*x.payload_mut()) = z.clone();
        //                                         z.apply_node_transform(x);

        //                                         let curr_edgs = kasten_guard
        //                                             .graph
        //                                             .g()
        //                                             .edges(gid)
        //                                             .map(|e| e.weight().id())
        //                                             .collect::<Vec<_>>();

        //                                         for edge in curr_edgs {
        //                                             let _ = kasten_guard.graph.remove_edge(edge);
        //                                         }

        //                                         for link in z.links {
        //                                             let dest = *kasten_guard
        //                                                 .zid_to_gid
        //                                                 .get(&link.dest)
        //                                                 .expect("must exist");
        //                                             kasten_guard.graph.add_edge(gid, dest, link);
        //                                         }

        //                                         info!(
        //                                             "kasten_guard.graph: {:#?} ",
        //                                             kasten_guard.graph
        //                                         );
        //                                     }
        //                                 }
        //                             }
        //                             Err(e) => error!("watch error: {:#?}", e),
        //                         }
        //                     }

        //                     Ok(())
        //                 }));
        //             }
        //         }
        //     });
        // });

        Self {
            file_dialog: FileDialog::new(),
            picked_file: None,
            kasten_bind: Bind::default(),
            kasten_watcher_bind: Bind::default(),
            kasten_sender_bind: Bind::default(),
            kasten_watcher_handle: kasten_watcher,
            curr_kasten_id: None,
            kasten_sender: tx,
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
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        ctx.plugin_or_default::<EguiAsyncPlugin>(); // <-- REQUIRED

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            if ui.button("Pick file").clicked() {
                // Open the file dialog to pick a file.
                self.file_dialog.pick_directory();
            }

            ui.label(format!("Picked directory: {:?}", self.picked_file));

            // Update the dialog
            self.file_dialog.update(ctx);

            // Check if the user picked a file.
            if let Some(path) = self.file_dialog.take_picked() {
                self.picked_file = Some(path.to_path_buf());
                self.kasten_bind.clear();
                self.kasten_bind
                    .request(async { Kasten::parse(path).await.map(|k| Arc::new(Mutex::new(k))) });
            }

            //     egui::MenuBar::new().ui(ui, |ui| {
            //         // NOTE: no File->Quit on web pages!
            //         let is_web = cfg!(target_arch = "wasm32");
            //         if !is_web {
            //             ui.menu_button("File", |ui| {
            //                 if ui.button("Quit").clicked() {
            //                     ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            //                 }
            //             });
            //             ui.add_space(16.0);
            //         }

            //         egui::widgets::global_theme_preference_buttons(ui);
            //     });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Emergence");

            // ui.horizontal(|ui| {
            //     ui.label("Write something: ");
            //     ui.text_edit_singleline(&mut self.label);
            // });

            // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     self.value += 1.0;
            // }

            // ui.separator();
            //

            match self.kasten_bind.read_mut() {
                Some(Ok(k)) => {
                    let mut kg = k.lock().expect("should never be poisoned");
                    match self.curr_kasten_id {
                        Some(ref mut id) => {
                            if *id != kg.id {
                                let sender = self.kasten_sender.clone();
                                let k_clone = k.clone();
                                self.kasten_sender_bind.clear();
                                self.kasten_sender_bind.request(async move {
                                    sender.send(k_clone).await.expect("lol");
                                    Ok(())
                                });

                                // .expect("this sender queue should never be full");

                                info!("sending kasten to watcher thread: {:#?}", kg.id);
                                self.curr_kasten_id = Some(kg.id.clone());
                            }
                        }
                        None => {
                            self.curr_kasten_id = Some(kg.id.clone());
                            let sender = self.kasten_sender.clone();
                            let k_clone = k.clone();
                            self.kasten_sender_bind.clear();
                            self.kasten_sender_bind.request(async move {
                                sender.send(k_clone).await.expect("lol");
                                Ok(())
                            });
                        }
                    };

                    let mut x = k.clone();

                    // self.kasten_watcher_bind
                    //     .request(async move { x.watch().await });

                    let g = &mut kg.graph;

                    type L = egui_graphs::LayoutForceDirected<
                        egui_graphs::FruchtermanReingoldWithCenterGravity,
                    >;
                    type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;
                    let mut view = egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::new(g);
                    ui.add(&mut view);
                }

                Some(Err(e)) => {
                    ui.label(format!("error with selecting a ZettelKasten: {:?}", e));
                }

                None => {
                    ui.label("please choose a zettelkasten".to_string());
                }
            }

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
