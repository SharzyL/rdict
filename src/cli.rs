use crate::api;
use crate::config::Config;
use anyhow::Result;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;
use termimad::MadSkin;
use termimad::crossterm::style::{Color, Stylize};

pub fn run(config: Config, word: String) -> Result<()> {
    if word.trim().is_empty() {
        anyhow::bail!("No word provided. Usage: rdict --cli <word>");
    }

    if !config.api.api_key.starts_with("sk-") {
        anyhow::bail!("API key not configured. Please edit ~/.config/rdict/config.toml");
    }

    // Print header
    println!("{} {}\n", "Querying:".magenta(), word.clone().bold());

    let (sender, receiver) = channel();
    let cancel_token = Arc::new(AtomicBool::new(false));
    api::query_word_stream(config, word, sender, cancel_token);

    let skin = make_skin();
    let mut printed_lines = 0;
    let mut last_content = String::new();
    let mut stdout = io::stdout();

    // Receive streaming updates and print complete lines with color
    while let Ok(result) = receiver.recv() {
        match result {
            Ok(content) => {
                last_content = content;
                let lines: Vec<&str> = last_content.lines().collect();
                let complete_lines = if last_content.ends_with('\n') {
                    lines.len()
                } else {
                    lines.len().saturating_sub(1)
                };

                // Print newly completed lines with formatting
                for line in lines
                    .iter()
                    .skip(printed_lines)
                    .take(complete_lines - printed_lines)
                {
                    print_md_line(&skin, line);
                }
                printed_lines = complete_lines;

                stdout.flush()?;
            }
            Err(e) => {
                eprintln!("\nError: {}", e);
                return Err(e);
            }
        }
    }

    // Print any remaining partial line from the last content
    let lines: Vec<&str> = last_content.lines().collect();
    for line in lines.iter().skip(printed_lines) {
        print_md_line(&skin, line);
    }

    println!();

    Ok(())
}

fn print_md_line(skin: &MadSkin, line: &str) {
    // Render single line as markdown text (preserves headers, bold, etc.)
    let formatted = skin.text(line, None);
    print!("{}", formatted);
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default_dark();

    // Customize colors for headers
    skin.headers[0].set_fg(Color::Magenta);
    skin.headers[1].set_fg(Color::Cyan);
    skin.headers[2].set_fg(Color::Blue);

    // Bold text
    skin.bold.set_fg(Color::Yellow);

    // Italic text
    skin.italic.set_fg(Color::Green);

    // Code/inline code
    skin.inline_code.set_fg(Color::DarkYellow);
    skin.inline_code.set_bg(Color::DarkGrey);

    // Bullet points
    skin.bullet = termimad::StyledChar::from_fg_char(Color::Cyan, '•');

    skin
}
