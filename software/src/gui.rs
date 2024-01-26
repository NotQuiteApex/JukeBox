use eframe::egui;
use egui::{Color32, RichText, Separator};

pub fn basic_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 320.0])
            .with_maximize_button(false)
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_simple_native("JukeBox Desktop", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label(
                RichText::new("JukeBox Desktop")
                    .heading()
                    .strong()
                    .color(Color32::from_rgb(255, 200, 100)),
            );

            ui.separator();

            ui.horizontal_top(|ui| {
                ui.columns(2, |c| {
                    c[0].columns(2, |c| {
                        c[0].separator();
                        c[0].label("CPU: ");
                        c[0].label("CPU Freq: ");
                        c[0].label("CPU Load: ");
                        c[0].label("CPU Temp: ");
                        c[0].separator();
                        c[0].label("GPU: ");
                        c[0].label("GPU Core Freq: ");
                        c[0].label("GPU Core Load: ");
                        c[0].label("GPU VRAM Freq: ");
                        c[0].label("GPU VRAM Load: ");
                        c[0].label("GPU Temp: ");
                        c[0].separator();
                        c[0].label("Memory: ");
                        c[0].separator();
                        
                        c[1].separator();
                        c[1].label("(N/A)");
                        c[1].label("(N/A)");
                        c[1].label("(N/A)");
                        c[1].label("(N/A)");
                        c[1].separator();
                        c[1].label("(N/A)");
                        c[1].label("(N/A)");
                        c[1].label("(N/A)");
                        c[1].label("(N/A)");
                        c[1].label("(N/A)");
                        c[1].label("(N/A)");
                        c[1].separator();
                        c[1].label("(N/A)");
                        c[1].separator();
                    });
                    
                    if c[1].button("Set RGB to red").clicked() {
                        println!("you shouldnt have done that");
                    }
                    if c[1].button("Update JukeBox").clicked() {
                        println!("Updating JukeBox...");
                    }
                });
            });

            ui.separator();
        });
    })
    .expect("eframe error");
}
