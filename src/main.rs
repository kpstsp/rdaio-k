mod symphonia_play;
mod symphonia_control;
mod tests;
use symphonia_play::play_mp3_with_symphonia;
use symphonia_control::PlaybackControl;
use id3::TagLike;
use rand::seq::SliceRandom;

use crossterm::{event, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, Terminal, widgets::{Block, Borders, List, ListItem, Paragraph, ListState}, layout::{Layout, Constraint, Direction}, style::{Style, Modifier, Color}};
use std::{io, error::Error, fs, path::PathBuf};
use std::env;

const QUEUE_FILE: &str = ".rdaio_queue";

pub fn save_queue(files: &[String], directory: &str) -> Result<(), Box<dyn Error>> {
    let mut queue_data = String::new();
    queue_data.push_str(directory);
    queue_data.push('\n');
    for file in files {
        queue_data.push_str(file);
        queue_data.push('\n');
    }
    fs::write(QUEUE_FILE, queue_data)?;
    Ok(())
}

pub fn load_queue() -> Result<Option<(String, Vec<String>)>, Box<dyn Error>> {
    if let Ok(content) = fs::read_to_string(QUEUE_FILE) {
        let mut lines = content.lines();
        if let Some(directory) = lines.next() {
            let files: Vec<String> = lines.map(|s| s.to_string()).collect();
            return Ok(Some((directory.to_string(), files)));
        }
    }
    Ok(None)
}

pub fn load_mp3_files(directory: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mp3_files: Vec<String> = fs::read_dir(directory)?
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
    Ok(mp3_files)
}

pub fn get_mp3_title(file_path: &str) -> Option<String> {
    if let Ok(tag) = id3::Tag::read_from_path(file_path) {
        if let Some(title) = tag.title() {
            return Some(title.to_string());
        }
    }
    None
}

pub fn get_display_name(file_name: &str, directory: &str, show_title: bool) -> String {
    if !show_title {
        return file_name.to_string();
    }
    
    let mut path = PathBuf::from(directory);
    path.push(file_name);
    
    match get_mp3_title(path.to_string_lossy().as_ref()) {
        Some(title) => title,
        None => file_name.to_string(),
    }
}

pub fn get_folder_contents(directory: &str) -> Result<Vec<(String, bool)>, Box<dyn Error>> {
    let mut items = vec![
        (String::from(".."), true),
        (String::from("."), true)
    ];
    let entries = fs::read_dir(directory)?;
    
    let mut folders = Vec::new();
    let mut files = Vec::new();
    
    for entry in entries {
        if let Ok(e) = entry {
            let path = e.path();
            if let Some(name) = path.file_name() {
                if let Some(item_name) = name.to_str() {
                    if path.is_dir() {
                        folders.push((item_name.to_string(), true));
                    } else if let Some(ext) = path.extension() {
                        if ext.eq_ignore_ascii_case("mp3") {
                            files.push((item_name.to_string(), false));
                        }
                    }
                }
            }
        }
    }
    
    folders.sort_by(|a, b| a.0.cmp(&b.0));
    files.sort_by(|a, b| a.0.cmp(&b.0));
    
    items.extend(folders);
    items.extend(files);
    Ok(items)
}

fn browse_folders(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, _debug_mode: bool) -> Result<String, Box<dyn Error>> {
    let mut current_path = String::from(".");
    let mut folder_state = ListState::default();

    loop {
        let contents = get_folder_contents(&current_path)?;
        let folder_items: Vec<ListItem> = contents.iter()
            .map(|(name, is_dir)| {
                let prefix = if *is_dir { "[D] " } else { "[F] " };
                ListItem::new(format!("{}{}", prefix, name))
            })
            .collect();

        if folder_state.selected().is_none() {
            folder_state.select(Some(0));
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Min(10),
                    Constraint::Length(4),
                ].as_ref())
                .split(f.size());

            let folder_list = List::new(folder_items.clone())
                .block(Block::default().borders(Borders::ALL).title(format!("Browse Folders - {}", current_path)))
                .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(folder_list, chunks[0], &mut folder_state);

            let help = Paragraph::new("[Up/Down] Navigate  [Enter] Open Dir  [L] Load Files  [ESC] Cancel")
                .block(Block::default().borders(Borders::ALL).title("Controls"));
            f.render_widget(help, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    event::KeyCode::Esc => {
                        return Err("Cancelled".into());
                    },
                    event::KeyCode::Down => {
                        let i = match folder_state.selected() {
                            Some(i) => {
                                if i >= contents.len() - 1 { 0 } else { i + 1 }
                            },
                            None => 0,
                        };
                        folder_state.select(Some(i));
                    },
                    event::KeyCode::Up => {
                        let i = match folder_state.selected() {
                            Some(i) => {
                                if i == 0 { contents.len() - 1 } else { i - 1 }
                            },
                            None => 0,
                        };
                        folder_state.select(Some(i));
                    },
                    event::KeyCode::Enter => {
                        if let Some(idx) = folder_state.selected() {
                            if let Some((name, is_dir)) = contents.get(idx) {
                                if *is_dir {
                                    if name == ".." {
                                        let mut path = if current_path == "." {
                                            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                                        } else {
                                            PathBuf::from(&current_path)
                                        };
                                        if path.pop() {
                                            current_path = path.to_string_lossy().to_string();
                                        }
                                        folder_state.select(Some(0));
                                    } else if name != "." {
                                        let mut path = if current_path == "." {
                                            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
                                        } else {
                                            PathBuf::from(&current_path)
                                        };
                                        path.push(name);
                                        current_path = path.to_string_lossy().to_string();
                                        folder_state.select(Some(0));
                                    }
                                }
                            }
                        }
                    },
                    event::KeyCode::Char('l') | event::KeyCode::Char('L') => {
                        if let Some(idx) = folder_state.selected() {
                            if let Some((name, is_dir)) = contents.get(idx) {
                                if *is_dir && name != "." && name != ".." {
                                    let mut path = PathBuf::from(&current_path);
                                    path.push(name);
                                    return Ok(path.to_string_lossy().to_string());
                                } else if *is_dir {
                                    return Ok(current_path);
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let debug_mode = args.contains(&"--debug".to_string());

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // List real MP3 files in the current directory
    let mut current_directory = String::from(".");
    let mut mp3_files = load_mp3_files(&current_directory)?;
    
    // Try to load saved queue
    if let Ok(Some((saved_dir, saved_files))) = load_queue() {
        if !saved_files.is_empty() {
            current_directory = saved_dir;
            mp3_files = saved_files;
        }
    }
    
    let mut state = ListState::default();
    if !mp3_files.is_empty() {
        state.select(Some(0));
    }
    let mut running = true;
    let mut symphonia_ctrl: Option<PlaybackControl> = None;
    let mut _symphonia_thread: Option<std::thread::JoinHandle<()>> = None;
    let mut current_playing_idx: Option<usize> = None;
    let mut show_title = true;
    let original_mp3_files = mp3_files.clone();
    
    while running {
        // Auto-play next song if current finished
        if let Some(ctrl) = &symphonia_ctrl {
            if ctrl.is_stopped() && current_playing_idx.is_some() {
                let current_idx = current_playing_idx.unwrap();
                if current_idx + 1 < mp3_files.len() {
                    // Play next song
                    let next_idx = current_idx + 1;
                    if let Some(file) = mp3_files.get(next_idx) {
                        if debug_mode {
                            println!("[DEBUG] Auto-playing next track: {}", file);
                        }
                        let new_ctrl = PlaybackControl::new();
                        let fname = if current_directory == "." {
                            file.clone()
                        } else {
                            let mut path = PathBuf::from(&current_directory);
                            path.push(file);
                            path.to_string_lossy().to_string()
                        };
                        let handle = std::thread::spawn({
                            let ctrl = new_ctrl.clone();
                            move || {
                                let _ = play_mp3_with_symphonia(&fname, ctrl);
                            }
                        });
                        symphonia_ctrl = Some(new_ctrl);
                        _symphonia_thread = Some(handle);
                        current_playing_idx = Some(next_idx);
                    }
                } else {
                    current_playing_idx = None;
                }
            }
        }
        
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Min(5),
                    Constraint::Length(4),
                ].as_ref())
                .split(f.size());

            let display_items: Vec<ListItem> = mp3_files.iter()
                .map(|f| ListItem::new(get_display_name(f, &current_directory, show_title)))
                .collect();
            
            let mode_str = if show_title { "Title" } else { "Filename" };
            let files_list = List::new(display_items)
                .block(Block::default().borders(Borders::ALL).title(format!("MP3 Files [{}] - {}", mode_str, current_directory)))
                .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .highlight_symbol("▶ ");
            f.render_stateful_widget(files_list, chunks[0], &mut state);

            let controls = Paragraph::new("Controls: [Up/Down] Select  [P] Play  [Z] Pause/Resume  [S] Stop  [PgUp/PgDn] Prev/Next  [M] Mode  [H] Shuffle  [O] Original  [F] Folder  [C] Clear  [Q] Quit")
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
                        if debug_mode {
                            println!("[DEBUG] Quit pressed");
                        }
                        running = false;
                    },
                    event::KeyCode::Char('m') | event::KeyCode::Char('M') => {
                        show_title = !show_title;
                        if debug_mode {
                            let mode = if show_title { "Title" } else { "Filename" };
                            println!("[DEBUG] Display mode switched to: {}", mode);
                        }
                    },
                    event::KeyCode::Char('h') | event::KeyCode::Char('H') => {
                        // Shuffle
                        let mut rng = rand::thread_rng();
                        mp3_files.shuffle(&mut rng);
                        state.select(if !mp3_files.is_empty() { Some(0) } else { None });
                        current_playing_idx = None;
                        if debug_mode {
                            println!("[DEBUG] Shuffled {} tracks", mp3_files.len());
                        }
                    },
                    event::KeyCode::Char('o') | event::KeyCode::Char('O') => {
                        // Restore original order
                        mp3_files = original_mp3_files.clone();
                        state.select(if !mp3_files.is_empty() { Some(0) } else { None });
                        current_playing_idx = None;
                        if debug_mode {
                            println!("[DEBUG] Restored original track order");
                        }
                    },
                    event::KeyCode::Char('f') | event::KeyCode::Char('F') => {
                        if debug_mode {
                            println!("[DEBUG] Folder browser requested");
                        }
                        match browse_folders(&mut terminal, debug_mode) {
                            Ok(selected_folder) => {
                                current_directory = selected_folder;
                                if let Ok(new_files) = load_mp3_files(&current_directory) {
                                    mp3_files = new_files;
                                    state.select(if !mp3_files.is_empty() { Some(0) } else { None });
                                    let _ = save_queue(&mp3_files, &current_directory);
                                    if debug_mode {
                                        println!("[DEBUG] Loaded {} files from {}", mp3_files.len(), current_directory);
                                    }
                                }
                            },
                            Err(_) => {
                                if debug_mode {
                                    println!("[DEBUG] Folder selection cancelled");
                                }
                            }
                        }
                    },
                    event::KeyCode::Char('c') | event::KeyCode::Char('C') => {
                        if debug_mode {
                            println!("[DEBUG] Clear queue pressed");
                        }
                        mp3_files.clear();
                        state.select(None);
                        current_playing_idx = None;
                        let _ = save_queue(&mp3_files, &current_directory);
                    },
                    event::KeyCode::Down => {
                        let i = match state.selected() {
                            Some(i) => {
                                if i >= mp3_files.len() - 1 { 0 } else { i + 1 }
                            },
                            None => 0,
                        };
                        state.select(Some(i));
                        if debug_mode {
                            println!("[DEBUG] Down pressed, selected index: {}", i);
                        }
                    },
                    event::KeyCode::Up => {
                        let i = match state.selected() {
                            Some(i) => {
                                if i == 0 { mp3_files.len() - 1 } else { i - 1 }
                            },
                            None => 0,
                        };
                        state.select(Some(i));
                        if debug_mode {
                            println!("[DEBUG] Up pressed, selected index: {}", i);
                        }
                    },
                    event::KeyCode::Char('s') | event::KeyCode::Char('S') => {
                        if let Some(ctrl) = &symphonia_ctrl {
                            if debug_mode {
                                println!("[DEBUG] Symphonia STOP");
                            }
                            ctrl.stop();
                        }
                    },
                    event::KeyCode::Char('p') | event::KeyCode::Char('P') => {
                        if let Some(idx) = state.selected() {
                            if let Some(file) = mp3_files.get(idx) {
                                if debug_mode {
                                    println!("[DEBUG] Symphonia playback: {}", file);
                                }
                                if let Some(ctrl) = &symphonia_ctrl {
                                    ctrl.stop();
                                }
                                let ctrl = PlaybackControl::new();
                                let fname = if current_directory == "." {
                                    file.clone()
                                } else {
                                    format!("{}\\{}", current_directory, file)
                                };
                                let handle = std::thread::spawn({
                                    let ctrl = ctrl.clone();
                                    move || {
                                        let _ = play_mp3_with_symphonia(&fname, ctrl);
                                    }
                                });
                                symphonia_ctrl = Some(ctrl);
                                _symphonia_thread = Some(handle);
                                current_playing_idx = Some(idx);
                            }
                        }
                    },
                    // Pause/Resume for Symphonia
                    event::KeyCode::Char('z') | event::KeyCode::Char('Z') => {
                        if let Some(ctrl) = &symphonia_ctrl {
                            if ctrl.is_paused() {
                                if debug_mode {
                                    println!("[DEBUG] Symphonia Resume (Z)");
                                }
                                ctrl.resume();
                            } else {
                                if debug_mode {
                                    println!("[DEBUG] Symphonia Pause (Z)");
                                }
                                ctrl.pause();
                            }
                        } else if debug_mode {
                            println!("[DEBUG] No symphonia playback");
                        }
                    },
                    event::KeyCode::PageDown => {
                        // Play next track
                        if !mp3_files.is_empty() {
                            let next_idx = match current_playing_idx {
                                Some(idx) => {
                                    if idx >= mp3_files.len() - 1 { 0 } else { idx + 1 }
                                },
                                None => match state.selected() {
                                    Some(i) => {
                                        if i >= mp3_files.len() - 1 { 0 } else { i + 1 }
                                    },
                                    None => 0,
                                }
                            };
                            
                            if let Some(file) = mp3_files.get(next_idx) {
                                if debug_mode {
                                    println!("[DEBUG] PageDown pressed - Play next track: {}", file);
                                }
                                state.select(Some(next_idx));
                                
                                if let Some(ctrl) = &symphonia_ctrl {
                                    ctrl.stop();
                                }
                                
                                let ctrl = PlaybackControl::new();
                                let fname = if current_directory == "." {
                                    file.clone()
                                } else {
                                    format!("{}\\{}", current_directory, file)
                                };
                                let handle = std::thread::spawn({
                                    let ctrl = ctrl.clone();
                                    move || {
                                        let _ = play_mp3_with_symphonia(&fname, ctrl);
                                    }
                                });
                                symphonia_ctrl = Some(ctrl);
                                _symphonia_thread = Some(handle);
                                current_playing_idx = Some(next_idx);
                            }
                        }
                    },
                    event::KeyCode::PageUp => {
                        // Play previous track
                        if !mp3_files.is_empty() {
                            let prev_idx = match current_playing_idx {
                                Some(idx) => {
                                    if idx == 0 { mp3_files.len() - 1 } else { idx - 1 }
                                },
                                None => match state.selected() {
                                    Some(i) => {
                                        if i == 0 { mp3_files.len() - 1 } else { i - 1 }
                                    },
                                    None => mp3_files.len() - 1,
                                }
                            };
                            
                            if let Some(file) = mp3_files.get(prev_idx) {
                                if debug_mode {
                                    println!("[DEBUG] PageUp pressed - Play previous track: {}", file);
                                }
                                state.select(Some(prev_idx));
                                
                                if let Some(ctrl) = &symphonia_ctrl {
                                    ctrl.stop();
                                }
                                
                                let ctrl = PlaybackControl::new();
                                let fname = if current_directory == "." {
                                    file.clone()
                                } else {
                                    format!("{}\\{}", current_directory, file)
                                };
                                let handle = std::thread::spawn({
                                    let ctrl = ctrl.clone();
                                    move || {
                                        let _ = play_mp3_with_symphonia(&fname, ctrl);
                                    }
                                });
                                symphonia_ctrl = Some(ctrl);
                                _symphonia_thread = Some(handle);
                                current_playing_idx = Some(prev_idx);
                            }
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
