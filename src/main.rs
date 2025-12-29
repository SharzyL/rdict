mod api;
mod app;
mod config;
mod fonts;

use anyhow::Result;
use arboard::Clipboard;
use clap::Parser;
use eframe::egui;
use fonts::setup_fonts;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "RDict")]
#[command(about = "AI-powered dictionary with GUI", long_about = None)]
struct Args {
    /// Path to config file (defaults to ~/.config/rdict/config.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Suppress info and debug logs, only show warnings and errors
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate conflicting arguments
    if args.debug && args.quiet {
        anyhow::bail!("Cannot use both --debug and --quiet flags");
    }

    // Determine logging level
    let log_level = if args.quiet {
        tracing::Level::WARN
    } else if args.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    // Initialize tracing subscriber
    tracing_subscriber::fmt().with_max_level(log_level).init();

    // Load configuration
    let config = config::Config::load_from_path(args.config)?;

    // Get word from clipboard
    let clipboard_text = Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get_text().ok())
        .unwrap_or_default()
        .trim()
        .to_string();

    // Create application
    let app = app::RDictApp::new(config, clipboard_text);

    // Run GUI
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RDict",
        options,
        Box::new(|cc| {
            // Setup Chinese font support
            setup_fonts(&cc.egui_ctx);

            // Set larger font sizes
            let mut style = (*cc.egui_ctx.style()).clone();
            style.text_styles = [
                (
                    egui::TextStyle::Heading,
                    egui::FontId::new(28.0, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Body,
                    egui::FontId::new(18.0, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Monospace,
                    egui::FontId::new(16.0, egui::FontFamily::Monospace),
                ),
                (
                    egui::TextStyle::Button,
                    egui::FontId::new(16.0, egui::FontFamily::Proportional),
                ),
                (
                    egui::TextStyle::Small,
                    egui::FontId::new(14.0, egui::FontFamily::Proportional),
                ),
            ]
            .into();
            cc.egui_ctx.set_style(style);

            Ok(Box::new(app))
        }),
    )
    .map_err(|e| anyhow::anyhow!("Failed to run app: {}", e))
}
