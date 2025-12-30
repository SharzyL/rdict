use crate::api;
use crate::config::Config;
use anyhow::Result;
use eframe::egui;
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use tracing::{debug, warn};

pub struct RDictApp {
    config: Config,
    input_word: String,
    result: String,
    is_loading: bool,
    is_debug: bool,
    error_message: Option<String>,
    result_receiver: Option<Receiver<Result<String>>>,
    cancel_token: Option<Arc<AtomicBool>>,
    markdown_cache: CommonMarkCache,
    first_frame: bool,
}

impl RDictApp {
    pub fn new(config: Config, is_debug: bool, initial_word: String) -> Self {
        let mut app = Self {
            config,
            input_word: initial_word.clone(),
            result: String::new(),
            is_loading: false,
            is_debug,
            error_message: None,
            result_receiver: None,
            cancel_token: None,
            markdown_cache: CommonMarkCache::default(),
            first_frame: true,
        };

        // If clipboard has content on startup, auto-query
        if !initial_word.is_empty() && app.config.api.api_key != "your-api-key-here" {
            debug!("Clipboard content detected: {}", initial_word);
            app.start_query();
        } else if initial_word.is_empty() {
            debug!("Clipboard is empty");
        } else {
            debug!("API key not configured, skipping auto-query");
        }

        app
    }

    fn start_query(&mut self) {
        if self.input_word.trim().is_empty() {
            self.error_message = Some("Enter query".to_string());
            return;
        }

        if self.config.api.api_key == "your-api-key-here" {
            self.error_message = Some("Configure API Key".to_string());
            return;
        }

        // Cancel previous query if it exists
        if let Some(token) = &self.cancel_token {
            debug!("Cancelling previous query");
            token.store(true, Ordering::Relaxed);
        }

        self.is_loading = true;
        self.error_message = None;
        self.result.clear();

        debug!("Starting query for: {}", self.input_word);

        let (sender, receiver): (Sender<Result<String>>, Receiver<Result<String>>) = channel();
        self.result_receiver = Some(receiver);

        // Create new cancel token for this query
        let cancel_token = Arc::new(AtomicBool::new(false));
        self.cancel_token = Some(cancel_token.clone());

        api::query_word_stream(
            self.config.clone(),
            self.input_word.clone(),
            sender,
            cancel_token,
        );
    }

    fn check_result(&mut self) {
        if let Some(receiver) = &self.result_receiver {
            // Try to receive all available messages (for streaming)
            loop {
                match receiver.try_recv() {
                    Ok(result) => {
                        match result {
                            Ok(content) => {
                                self.result = content;
                                self.error_message = None;
                                // Keep is_loading true to continue receiving updates
                            }
                            Err(e) => {
                                warn!("Query failed: {}", e);
                                self.error_message = Some(format!("Query failed: {}", e));
                                self.is_loading = false;
                                self.result_receiver = None;
                                break;
                            }
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // No more messages available right now
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // Stream ended normally
                        self.is_loading = false;
                        self.result_receiver = None;
                        break;
                    }
                }
            }
        }
    }
}

impl eframe::App for RDictApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check async results
        self.check_result();

        // Request repaint if loading
        if self.is_loading {
            ctx.request_repaint();
        }

        if self.is_debug {
            #[cfg(debug_assertions)]
            ctx.set_debug_on_hover(true);
        }

        // Close app on Escape key
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Main panel
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(12)))
            .show(ctx, |ui| {
                // Input area
                ui.horizontal(|ui| {
                    let text_edit = egui::TextEdit::singleline(&mut self.input_word)
                        .hint_text("Enter word to query...");

                    let query_box = ui.add_sized([ui.available_width() - 70.0, 24.0], text_edit);

                    // Request focus on first frame
                    if self.first_frame {
                        query_box.request_focus();
                        self.first_frame = false;
                    }

                    // Query on Enter key
                    if query_box.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.start_query();
                    }

                    let button_enabled = !self.input_word.trim().is_empty();
                    if ui
                        .add_enabled(button_enabled, egui::Button::new("Query"))
                        .clicked()
                    {
                        self.start_query();
                    }
                });

                // Error message
                if let Some(error) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                }

                ui.separator();

                // Result display area
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if !self.result.is_empty() {
                            CommonMarkViewer::new().show(
                                ui,
                                &mut self.markdown_cache,
                                &self.result,
                            );
                        } else if !self.is_loading && self.error_message.is_none() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(50.0);
                                ui.label("Enter a word and click Query, or press Enter");
                            });
                        }
                        if self.is_loading {
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Generating...");
                            });
                        }
                    });
            });
    }
}
