mod symphonia_play;
mod symphonia_control;
use symphonia_play::play_mp3_with_symphonia;
use symphonia_control::PlaybackControl;

use crossterm::{event, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, List, ListItem, Paragraph, ListState}, layout::{Layout, Constraint, Direction}, style::{Style, Modifier, Color}};
use std::{io, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;


    // List real MP3 files in the current directory
    let mp3_files: Vec<String> = fs::read_dir(".")?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.eq_ignore_ascii_case("mp3") {
                            return path.file_name().and_then(|n| n.to_str().map(|s| s.to_string()));
                        }
                    }
                }
                None
            })
        })
        .collect();
    let items: Vec<ListItem> = mp3_files.iter().map(|f| ListItem::new(f.as_str())).collect();

    let mut state = ListState::default();
    if !mp3_files.is_empty() {
        state.select(Some(0));
    }
    let mut running = true;
    let mut symphonia_ctrl: Option<PlaybackControl> = None;
    let mut _symphonia_thread: Option<std::thread::JoinHandle<()>> = None;
    while running {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Min(5),
                    Constraint::Length(3),
                ].as_ref())
                .split(f.size());

            let files_list = List::new(items.clone())
                .block(Block::default().borders(Borders::ALL).title("MP3 Files"))
                .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .highlight_symbol("▶ ");
            f.render_stateful_widget(files_list, chunks[0], &mut state);

            let controls = Paragraph::new("Controls: [Up/Down] Select  [P] Play  [Z] Pause/Resume  [S] Stop  [Q] Quit")
                .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(controls, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let event::Event::Key(key) = event::read()? {
                // Only handle KeyPress events, ignore KeyRelease and KeyRepeat
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    event::KeyCode::Char('q') | event::KeyCode::Char('Q') => {
                        println!("[DEBUG] Quit pressed");
                        running = false;
                    },
                    event::KeyCode::Down => {
                        let i = match state.selected() {
                            Some(i) => {
                                if i >= mp3_files.len() - 1 { 0 } else { i + 1 }
                            },
                            None => 0,
                        };
                        state.select(Some(i));
                        println!("[DEBUG] Down pressed, selected index: {}", i);
                    },
                    event::KeyCode::Up => {
                        let i = match state.selected() {
                            Some(i) => {
                                if i == 0 { mp3_files.len() - 1 } else { i - 1 }
                            },
                            None => 0,
                        };
                        state.select(Some(i));
                        println!("[DEBUG] Up pressed, selected index: {}", i);
                    },
                    event::KeyCode::Enter => {
                        // Ignored - use P for play
                    },
                    event::KeyCode::Char(' ') => {
                        // Ignored - use Z for pause/resume
                    },
                    event::KeyCode::Char('s') | event::KeyCode::Char('S') => {
                        if let Some(ctrl) = &symphonia_ctrl {
                            println!("[DEBUG] Symphonia STOP");
                            ctrl.stop();
                        }
                    },
                    event::KeyCode::Char('p') | event::KeyCode::Char('P') => {
                        if let Some(idx) = state.selected() {
                            if let Some(file) = mp3_files.get(idx) {
                                println!("[DEBUG] Symphonia playback: {}", file);
                                if let Some(ctrl) = &symphonia_ctrl {
                                    ctrl.stop();
                                }
                                let ctrl = PlaybackControl::new();
                                let fname = file.clone();
                                let handle = std::thread::spawn({
                                    let ctrl = ctrl.clone();
                                    move || {
                                        let _ = play_mp3_with_symphonia(&fname, ctrl);
                                    }
                                });
                                symphonia_ctrl = Some(ctrl);
                                _symphonia_thread = Some(handle);
                            }
                        }
                    },
                    // Pause/Resume for Symphonia
                    event::KeyCode::Char('z') | event::KeyCode::Char('Z') => {
                        if let Some(ctrl) = &symphonia_ctrl {
                            if ctrl.is_paused() {
                                println!("[DEBUG] Symphonia Resume (Z)");
                                ctrl.resume();
                            } else {
                                println!("[DEBUG] Symphonia Pause (Z)");
                                ctrl.pause();
                            }
                        } else {
                            println!("[DEBUG] No symphonia playback");
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
