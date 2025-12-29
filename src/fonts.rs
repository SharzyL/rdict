use eframe::egui;
use tracing::{debug, warn};

pub const DEFAULT_FONT_NAME: &str = "Inter";
pub const MONOSPACE_FONT_NAME: &str = "Cascadia Mono";
pub const CJK_FONT_PATTERNS: &[&str] = &["Noto Sans CJK SC", "Noto Sans CJK JP", "Noto Sans CJK"];

#[cfg(unix)]
fn find_proportional_font() -> Option<Vec<u8>> {
    use fontconfig::Fontconfig;

    let fc = Fontconfig::new()?;

    // Search for Inter font
    if let Some(font) = fc.find(DEFAULT_FONT_NAME, None) {
        let path = font.path;
        if let Ok(data) = std::fs::read(&path) {
            debug!("Loaded Inter font: {}", path.display());
            return Some(data);
        }
    }

    None
}

#[cfg(unix)]
fn find_monospace_font() -> Option<Vec<u8>> {
    use fontconfig::Fontconfig;

    let fc = Fontconfig::new()?;

    // Search for Cascadia Mono font
    if let Some(font) = fc.find(MONOSPACE_FONT_NAME, None) {
        let path = font.path;
        if let Ok(data) = std::fs::read(&path) {
            debug!(
                "Loaded monospace font ({}): {}",
                MONOSPACE_FONT_NAME,
                path.display()
            );
            return Some(data);
        }
    }

    None
}

#[cfg(unix)]
fn find_cjk_font() -> Option<Vec<u8>> {
    use fontconfig::Fontconfig;

    let fc = Fontconfig::new()?;

    // Search for CJK fonts in priority order
    for pattern in CJK_FONT_PATTERNS {
        if let Some(font) = fc.find(pattern, None) {
            let path = font.path;
            if let Ok(data) = std::fs::read(&path) {
                debug!("Loaded CJK font: {}", path.display());
                return Some(data);
            }
        }
    }

    // If specific fonts not found, try any font supporting Chinese
    if let Some(font) = fc.find("sans", Some("zh")) {
        let path = font.path;
        if let Ok(data) = std::fs::read(&path) {
            debug!("Loaded font (fallback): {}", path.display());
            return Some(data);
        }
    }

    None
}

pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Add CJK font for Chinese and Japanese character support
    if let Some(font_data) = find_cjk_font() {
        fonts.font_data.insert(
            "cjk".to_owned(),
            egui::FontData::from_owned(font_data).into(),
        );

        // Add CJK font as fallback for both families
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("cjk".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("cjk".to_owned());
    } else {
        warn!("No CJK font found, Chinese and Japanese characters may not display correctly");
    }

    // Load Inter font as proportional font
    if let Some(font_data) = find_proportional_font() {
        fonts.font_data.insert(
            "proportional".to_owned(),
            egui::FontData::from_owned(font_data).into(),
        );

        // Set Inter as the default font for proportional family
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "proportional".to_owned());
    } else {
        warn!(
            "Proportional font {} not found, using system default for proportional",
            DEFAULT_FONT_NAME
        );
    }

    // Load Cascadia Mono as monospace font
    if let Some(font_data) = find_monospace_font() {
        fonts.font_data.insert(
            "mono".to_owned(),
            egui::FontData::from_owned(font_data).into(),
        );

        // Set Cascadia Mono as the default font for monospace family
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "mono".to_owned());
    } else {
        warn!(
            "Mono font {} not found, using system default for monospace",
            MONOSPACE_FONT_NAME
        );
    }

    ctx.set_fonts(fonts);
}
