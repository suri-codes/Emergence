#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use emergence::EmergenceApp;

#[tokio::main]
async fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };

    eframe::run_native(
        "Emergence",
        native_options,
        Box::new(|cc| Ok(Box::new(EmergenceApp::new(cc)))),
    )
}

// use std::time::Duration;

// use eframe::egui;
// use egui_async::{Bind, EguiAsyncPlugin};

// struct MyApp {
//     /// The Bind struct holds the state of our async operation.
//     data_bind: Bind<String, String>,
// }

// impl Default for MyApp {
//     fn default() -> Self {
//         Self {
//             // We initialize the Bind and tell it to not retain data
//             // if it's not visible for a frame.
//             // If set to true, this will retain data even as the
//             // element goes undrawn.
//             data_bind: Bind::new(false), // Same as Bind::default()
//         }
//     }
// }

// impl eframe::App for MyApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         // This registers the plugin that drives the async event loop.
//         // It's idempotent and cheap to call on every frame.
//         //

//         ctx.plugin_or_default::<EguiAsyncPlugin>(); // <-- REQUIRED

//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.heading("Async Data Demo");
//             ui.add_space(10.0);

//             let ip_fn = || async {
//                 tokio::time::sleep(Duration::from_secs(2)).await;
//                 reqwest::get("https://icanhazip.com/")
//                     .await
//                     .map_err(|e| e.to_string())?
//                     .text()
//                     .await
//                     .map_err(|e| e.to_string())
//             };

//             // Request if `data_bind` is None and idle
//             // Otherwise, just read it
//             //
//             //
//             if ui.button("resend").clicked() {
//                 self.data_bind.refresh(ip_fn());
//             }

//             if let Some(res) = self.data_bind.read_or_request(ip_fn) {
//                 match res {
//                     Ok(ip) => {
//                         ui.label(format!("Your public IP is: {ip}"));
//                     }
//                     Err(err) => {
//                         ui.colored_label(
//                             egui::Color32::RED,
//                             format!("Could not fetch IP.\nError: {err}"),
//                         );
//                     }
//                 }
//             } else {
//                 ui.label("Getting public IP...");
//                 ui.spinner();
//             }
//         });
//     }
// }

// // Boilerplate
// fn main() {
//     let native_options = eframe::NativeOptions::default();
//     eframe::run_native(
//         "egui-async example",
//         native_options,
//         Box::new(|_cc| Ok(Box::new(MyApp::default()))),
//     )
//     .unwrap();
// }
