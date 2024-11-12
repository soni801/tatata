use std::path::PathBuf;
use std::process;
use clap::Parser;
use enigo::{Button, Coordinate, Enigo, Key, Keyboard, Mouse, Settings};
use enigo::Direction::Click;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// The TATATA file to execute
    file: PathBuf,

    /// If passed, print output to stdout instead of sending events
    #[arg(short, long, default_value_t = false)]
    dry_run: bool
}

#[derive(Debug)]
struct QueueItem {
    time: u64,
    actions: Vec<Action>
}

#[derive(Debug)]
enum Action {
    MouseMoveAction {
        x: i32,
        y: i32
    },
    MousePressAction {
        button: Button
    },
    KeyPressAction {
        key: Key
    }
}

fn main() {
    // Vec of action queue
    let mut queue: Vec<QueueItem> = Vec::new();

    // Get arguments
    let args = Arguments::parse();
    let dry_run = args.dry_run;
    let result = std::fs::read_to_string(&args.file);

    // Try to open file
    let script = match result {
        Ok(script) => script,
        Err(error) => {
            println!("Couldn't open input file for execution: {error}");
            process::exit(1);
        }
    };

    // Parse file
    let mut line_index = 0;
    for line in script.lines() {
        line_index += 1;

        // Skip if line is a comment or empty
        if line.starts_with("//") || line.is_empty() {
            continue;
        }

        // Get line data
        let line_decoded: Vec<&str> = line.split(">").collect();
        if line_decoded.len() != 2 {
            println!("Line {line_index}: Incorrectly formatted line: {line}");
            process::exit(1);
        }

        // Decode line
        let line_timestamp_text = line_decoded[0];
        let line_actions_text = line_decoded[1];
        let line_timestamp: u64 = line_timestamp_text.parse().unwrap_or_else(|error| {
            println!("Line {line_index}: Incorrectly formatted timestamp: {line_timestamp_text:?} ({error})");
            process::exit(1);
        });
        let line_actions: Vec<&str> = line_actions_text.split(";").collect();
        if line_actions.len() == 1 && line_actions[0].is_empty() {
            println!("Line {line_index}: Need at least one action");
            process::exit(1);
        }

        // Reject lines that have a timestamp lower than the previous line
        if let Some(previous_action) = queue.last() {
            if line_timestamp < previous_action.time {
                println!("Line {line_index}: Timestamp cannot be lower than previous action line");
                process::exit(1);
            }
        }

        // Store actions as Vec<Action>
        let mut actions: Vec<Action> = Vec::new();
        for mut action in line_actions {
            // Ignore empty actions
            action = action.trim();
            if action.is_empty() {
                continue;
            }

            // Add Action to actions
            if action.starts_with("mousemove") {
                // Validate arguments
                let segments: Vec<&str> = action.split_whitespace().collect();
                if segments.len() < 3 {
                    println!("Line {line_index} (mousemove): Too few arguments! (min. 2 arguments)");
                    process::exit(1);
                }
                if segments.len() > 3 {
                    println!("Line {line_index} (mousemove): Too many arguments provided (max. 2 arguments)");
                    process::exit(1);
                }

                // Parse X position
                let x: i32 = segments[1].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} (mousemove): Invalid X position {:?} ({error})", segments[1]);
                    process::exit(1);
                });

                // Parse Y position
                let y: i32 = segments[2].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} (mousemove): Invalid Y position {:?} ({error})", segments[2]);
                    process::exit(1);
                });

                // Add to actions
                actions.push(Action::MouseMoveAction { x, y });
            } else if action.starts_with("mousepress") {
                // Validate arguments
                let segments: Vec<&str> = action.split_whitespace().collect();
                if segments.len() < 2 {
                    println!("Line {line_index} (mousepress): No argument provided");
                    process::exit(1);
                }
                if segments.len() > 2 {
                    println!("Line {line_index} (mousepress): Too many arguments provided (max. 1 argument)");
                    process::exit(1);
                }

                // Parse button
                let button_number: u8 = segments[1].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} (mousepress): Invalid button {:?} ({error})", segments[1]);
                    process::exit(1);
                });

                let button = match button_number {
                    1 => Button::Left,
                    2 => Button::Right,
                    3 => Button::Middle,
                    #[cfg(not(target_os = "macos"))]
                    4 => Button::Back,
                    #[cfg(not(target_os = "macos"))]
                    5 => Button::Forward,
                    _ => {
                        println!("Line {line_index} (mousepress): Invalid button {:?}", segments[1]);
                        process::exit(1);
                    }
                };

                // Add to actions
                actions.push(Action::MousePressAction { button });
            } else if action.starts_with("keypress") {
                // Validate arguments
                let segments: Vec<&str> = action.split_whitespace().collect();
                if segments.len() < 2 {
                    println!("Line {line_index} (keypress): No argument provided");
                    process::exit(1);
                }
                if segments.len() > 2 {
                    println!("Line {line_index} (keypress): Too many arguments provided (max. 1 argument)");
                    process::exit(1);
                }

                // Parse key
                let key = match segments[1].to_lowercase().as_str() {
                    "alt" => Key::Alt,
                    "backspace" => Key::Backspace,
                    "capslock" => Key::CapsLock,
                    "control" => Key::Control,
                    "delete" => Key::Delete,
                    "down" => Key::DownArrow,
                    "end" => Key::End,
                    "enter" => Key::Return,
                    "escape" => Key::Escape,
                    "f1" => Key::F1,
                    "f2" => Key::F2,
                    "f3" => Key::F3,
                    "f4" => Key::F4,
                    "f5" => Key::F5,
                    "f6" => Key::F6,
                    "f7" => Key::F7,
                    "f8" => Key::F8,
                    "f9" => Key::F9,
                    "f10" => Key::F10,
                    "f11" => Key::F11,
                    "f12" => Key::F12,
                    "f13" => Key::F13,
                    "f14" => Key::F14,
                    "f15" => Key::F15,
                    "f16" => Key::F16,
                    "f17" => Key::F17,
                    "f18" => Key::F18,
                    "f19" => Key::F19,
                    "f20" => Key::F20,
                    "home" => Key::Home,
                    #[cfg(not(target_os = "macos"))]
                    "insert" => Key::Insert,
                    "left" => Key::LeftArrow,
                    "pagedown" => Key::PageDown,
                    "pageup" => Key::PageUp,
                    "right" => Key::RightArrow,
                    "shift" => Key::Shift,
                    "space" => Key::Space,
                    "super" => Key::Meta,
                    "tab" => Key::Tab,
                    "up" => Key::UpArrow,
                    _ => {
                        // Parse non-special keys
                        let key: char = segments[1].to_lowercase().parse().unwrap_or_else(|error| {
                            println!("Line {line_index} (keypress): Invalid key {:?} ({error})", segments[1]);
                            process::exit(1);
                        });

                        // Disallow non-standard keys
                        match key {
                            'a'..='z' => Key::Unicode(key),
                            '0'..='9' => Key::Unicode(key),
                            '`' => Key::Unicode(key),
                            '-' => Key::Unicode(key),
                            '=' => Key::Unicode(key),
                            '[' => Key::Unicode(key),
                            ']' => Key::Unicode(key),
                            '\\' => Key::Unicode(key),
                            ';' => Key::Unicode(key),
                            '\'' => Key::Unicode(key),
                            ',' => Key::Unicode(key),
                            '.' => Key::Unicode(key),
                            '/' => Key::Unicode(key),
                            _ => {
                                println!("Line {line_index} (keypress): Invalid key {:?}", segments[1]);
                                process::exit(1);
                            }
                        }
                    }
                };

                // Add to actions
                actions.push(Action::KeyPressAction { key });
            } else {
                println!("Line {line_index}: Invalid action: {action:?}");
                process::exit(1);
            }
        }

        // Add actions to queue
        queue.push(QueueItem {
            time: line_timestamp,
            actions
        });
    }

    // Execute file
    let mut current_timestamp = 0;
    let mut enigo = Enigo::new(&Settings::default()).unwrap_or_else(|error| {
        println!("Failed to initialize Enigo: {error}");
        process::exit(1);
    });

    for entry in queue {
        // Calculate wait time
        let wait_time = entry.time - current_timestamp;
        std::thread::sleep(std::time::Duration::from_millis(wait_time));

        // Execute actions
        for action in entry.actions {
            match action {
                Action::MouseMoveAction { x, y } => {
                    if dry_run {
                        println!("Move mouse to {x}, {y}");
                    } else {
                        let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                    }
                }
                Action::MousePressAction { button } => {
                    if dry_run {
                        println!("Press mouse {button:?}");
                    } else {
                        let _ = enigo.button(button, Click);
                    }
                }
                Action::KeyPressAction { key } => {
                    if dry_run {
                        println!("Press key {key:?}");
                    } else {
                        if let Err(error) = enigo.key(key, Click) {
                            println!("Failed to press key {key:?}: {error}");
                        }
                    }
                }
            }
        }

        // Update current timestamp
        current_timestamp = entry.time;
    }
}
