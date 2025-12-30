use eframe::egui;
use tracing::{debug, warn};

const DEFAULT_FONT: &str = "Inter";
const DEFAULT_MONO_FONT: &str = "Cascadia Mono";
const DEFAULT_CJK_FONTS: &[&str] = &["Source Han Sans CN", "Noto Sans CJK SC", "Noto Sans CJK JP"];

#[cfg(unix)]
fn load_font_by_name(name: &str, lang: Option<&str>) -> Option<Vec<u8>> {
    use fontconfig::Fontconfig;

    let fc = Fontconfig::new()?;
    let font = fc.find(name, lang)?;
    std::fs::read(&font.path).ok()
}

#[cfg(unix)]
fn find_proportional_font() -> Option<Vec<u8>> {
    load_font_by_name(DEFAULT_FONT, None).map(|data| {
        debug!("Loaded proportional font {}", DEFAULT_FONT);
        data
    })
}

#[cfg(unix)]
fn find_monospace_font() -> Option<Vec<u8>> {
    load_font_by_name(DEFAULT_MONO_FONT, None).map(|data| {
        debug!("Loaded monospace font ({})", DEFAULT_MONO_FONT);
        data
    })
}

#[cfg(unix)]
fn find_cjk_font() -> Option<Vec<u8>> {
    // Search for CJK fonts in priority order
    for pattern in DEFAULT_CJK_FONTS {
        if let Some(data) = load_font_by_name(pattern, None) {
            debug!("Loaded CJK font {}", pattern);
            return Some(data);
        }
    }

    // If specific fonts not found, try any font supporting Chinese
    if let Some(data) = load_font_by_name("sans", Some("zh")) {
        debug!("Loaded font (fallback)");
        return Some(data);
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
            .insert(0, "cjk".to_owned());

        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "cjk".to_owned());
    } else {
        warn!("No CJK font found, CJK characters may not display correctly");
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
            DEFAULT_FONT
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
            DEFAULT_MONO_FONT
        );
    }

    ctx.set_fonts(fonts);
}
