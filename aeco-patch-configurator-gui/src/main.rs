// Don't open a command prompt on Windows
#![windows_subsystem = "windows"]

use aeco_patch_config::{error::PatchConfigError, generate_config};
use eframe::egui;
use eframe::epaint::Vec2;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::{sync::mpsc, thread};
mod file_tree;
use file_tree::FileTree;
mod folder_picker;
use folder_picker::FolderPickWorker;

/// Messages which the worker thread (for generating configs) can send back to
/// the GUI about the result of the operation.
enum MessageToGUI {
    Complete,
    Error(PatchConfigError),
}

struct PatchConfigApp {
    patch_folder: String,
    patch_folder_picker: Option<FolderPickWorker>,
    patch_output_folder: String,
    patch_output_folder_picker: Option<FolderPickWorker>,
    state_message: String,
    worker_rx: Option<Receiver<MessageToGUI>>,
    file_tree: Option<FileTree>,
    maintenance_mode: bool,
}

impl PatchConfigApp {
    pub fn new() -> Self {
        Self {
            patch_folder: String::default(),
            patch_folder_picker: None,
            patch_output_folder: String::default(),
            patch_output_folder_picker: None,
            state_message: String::default(),
            worker_rx: None,
            file_tree: None,
            maintenance_mode: false,
        }
    }

    /// Starts a new thread to process a config generation task. Only one may
    /// be running at a given time.
    fn start_config_worker(&mut self, input_dir: &Path, output_dir: &Path) {
        // Do nothing if a worker is already processing data
        if self.worker_rx.is_some() {
            return;
        }

        self.set_message("Working...");

        let (tx_gui, rx_gui) = channel::<MessageToGUI>();

        // Keep the rx side of the channel to receive an update once the task
        // is finished
        self.worker_rx = Some(rx_gui);

        // Convert to Paths so the contents can be owned by the new thread
        let input_dir = input_dir.to_path_buf();
        let output_dir = output_dir.to_path_buf();

        let maintenance_mode = self.maintenance_mode;

        // Generate the configuration on a new thread
        thread::spawn(move || {
            let result = generate_config(input_dir, output_dir, maintenance_mode);

            // Send a response to the GUI depending on what the result of the
            // operation was
            let message = match result {
                Ok(_) => MessageToGUI::Complete,
                Err(why) => MessageToGUI::Error(why),
            };

            if let Err(why) = tx_gui.send(message) {
                eprintln!("Could not send worker response back to GUI: {why}");
            }
        });
    }

    /// If a config worker is running, check on its status and update the GUI
    /// if it has finished.
    fn check_config_worker(&mut self) {
        // Only check if a worker rx channel has been created
        if let Some(rx) = &self.worker_rx {
            let message = match rx.try_recv() {
                Ok(message) => message,
                Err(err) => match err {
                    mpsc::TryRecvError::Empty => return,
                    mpsc::TryRecvError::Disconnected => {
                        eprintln!("The worker channel has disconnected.");
                        return;
                    }
                },
            };

            // Provide feedback to the user depending on the result of the
            // operation
            match message {
                MessageToGUI::Complete => {
                    self.set_message("Finished!");
                }
                MessageToGUI::Error(why) => {
                    self.set_message(&format!("Failled to generate output: {}", why.to_string()));
                }
            }

            // Remove this end of the worker channel so new workers can be
            // created
            self.worker_rx = None;
        }
    }

    /// Sets the status message which is displayed to the user
    pub fn set_message(&mut self, message: &str) {
        self.state_message = message.to_string();
    }

    fn generate_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("Generate").clicked() {
            // Only start a config generation task if one is not already
            // running
            if self.worker_rx.is_none() {
                let mut output_dir = PathBuf::new();
                output_dir.push(&self.patch_output_folder);
                output_dir.push("aeco-patch");

                let input_dir = PathBuf::from(&self.patch_folder);
                self.start_config_worker(&input_dir, &output_dir);
            } else {
                self.set_message("Generation already in progress.")
            }
        }
    }

    /// Starts a file picker for the input folder on a new thread
    fn browse_patch_folder_button(&mut self, ui: &mut egui::Ui) {
        if !ui.button("Browse").clicked() {
            return;
        }

        if self.patch_folder_picker.is_none() {
            self.patch_folder_picker = Some(FolderPickWorker::start());
        }
    }

    /// Starts a file picker for the output folder on a new thread
    fn browse_patch_output_folder_button(&mut self, ui: &mut egui::Ui) {
        if !ui.button("Browse").clicked() {
            return;
        }

        if self.patch_output_folder_picker.is_none() {
            self.patch_output_folder_picker = Some(FolderPickWorker::start());
        }
    }

    /// Checks to see if there are responses from any file picker workers.
    /// Updates the paths in the GUI if there are responses.
    fn update_folders(&mut self) {
        // Get updated patch folders from any workers
        let mut updated_patch_folder: Option<PathBuf> = None;
        let mut updated_patch_output_folder: Option<PathBuf> = None;

        if let Some(worker) = &self.patch_folder_picker {
            if let Some(optional_path) = &worker.result() {
                updated_patch_folder = optional_path.clone();
                self.patch_folder_picker = None;
            }
        }

        if let Some(worker) = &self.patch_output_folder_picker {
            if let Some(optional_path) = &worker.result() {
                updated_patch_output_folder = optional_path.clone();
                self.patch_output_folder_picker = None;
            }
        }

        // Update displays
        if let Some(path) = updated_patch_folder {
            match path.to_str() {
                Some(path) => {
                    self.patch_folder = path.to_owned();
                    match FileTree::new(path) {
                        Ok(tree) => {
                            self.file_tree = Some(tree);
                        }
                        Err(_) => {
                            self.set_message("Failed to generate tree for selected input path.");
                        }
                    }
                }
                None => {
                    self.set_message("Selected path could not be converted to a string.");
                }
            }
        }

        if let Some(path) = updated_patch_output_folder {
            match path.to_str() {
                Some(path) => {
                    self.patch_output_folder = path.to_owned();
                }
                None => {
                    self.set_message("Selected path could not be converted to a string.");
                }
            }
        }
    }
}

impl eframe::App for PatchConfigApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(10));
        self.update_folders();
        self.check_config_worker();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::TopBottomPanel::top("top-panel").show_inside(ui, |ui| {
                egui::SidePanel::right("generate-panel")
                    .frame(egui::Frame::none())
                    .show_inside(ui, |ui| {
                        ui.checkbox(&mut self.maintenance_mode, "Maintenance");
                        ui.centered_and_justified(|ui| {
                            self.generate_button(ui);
                        });
                    });
                ui.label("Patch Folder");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.patch_folder);
                    self.browse_patch_folder_button(ui);
                });

                ui.label("Patch Output Folder");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.patch_output_folder);
                    self.browse_patch_output_folder_button(ui);
                });
            });
            egui::TopBottomPanel::top("message-panel").show_inside(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    egui::ScrollArea::new([true, false]).show_viewport(ui, |ui, _| {
                        ui.label(&self.state_message);
                    });
                });
            });

            if let Some(tree) = &mut self.file_tree {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    // While we aren't strictly working with rows, we need to
                    // tell the ScrollArea how tall it is, so we pretend we
                    // have 1 row at the current height of the tree.
                    .show_rows(ui, tree.height(), 1, |ui, _| {
                        tree.show(ui);
                    });
            }
        });
    }
}

fn main() {
    let initial_window_size = Vec2::new(600., 600.);
    let max_window_size = Vec2::new(initial_window_size.x, initial_window_size.y + 1000.);
    let min_window_size = Vec2::new(initial_window_size.x, initial_window_size.y - 400.);

    eframe::run_native(
        "AECO Patch Configurator",
        eframe::NativeOptions {
            // icon_data: todo!(),
            initial_window_size: Some(initial_window_size),
            max_window_size: Some(max_window_size),
            min_window_size: Some(min_window_size),
            resizable: true,
            ..eframe::NativeOptions::default()
        },
        Box::new(|_cc| Box::new(PatchConfigApp::new())),
    );
}
