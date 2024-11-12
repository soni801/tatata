use clap::Parser;
use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// The TATATA file to execute
    file: PathBuf,

    /// Print output to stdout instead of sending events
    #[arg(short, long, default_value_t = false)]
    dry_run: bool,

    /// Log all actions to stdout
    #[arg(short, long, default_value_t = false)]
    verbose: bool
}

#[derive(Debug)]
struct QueueItem {
    time: u64,
    actions: Vec<Action>
}

#[derive(Debug)]
enum Action {
    MouseMove {
        x: i32,
        y: i32
    },
    MouseDown {
        button: Button
    },
    MouseUp {
        button: Button
    },
    KeyDown {
        key: Key
    },
    KeyUp {
        key: Key
    }
}

fn main() {
    // Get arguments
    let args = Arguments::parse();
    let queue = parse_file(args.file);
    let dry_run = args.dry_run;
    let verbose = args.verbose;

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
            execute_action(&mut enigo, entry.time, action, !dry_run, dry_run || verbose);
        }

        // Update current timestamp
        current_timestamp = entry.time;
    }
}

fn parse_file(file_path: PathBuf) -> Vec<QueueItem> {
    // Check if file exists
    if !file_path.exists() {
        println!("File does not exist: {}", file_path.display());
        process::exit(1);
    }

    // Validate file name (https://github.com/soni801/tatata/issues/1)
    let file_name = file_path.to_str().unwrap_or_else(|| {
        println!("Invalid file name: {}", file_path.display());
        process::exit(1);
    });
    if !file_name.ends_with(".tatata") {
        println!("Not a TATATA file: {}", file_path.display());
        process::exit(1);
    }

    // Try to open file
    let file_content = std::fs::read_to_string(file_path).unwrap_or_else(|error| {
        println!("Couldn't open input file for execution: {error}");
        process::exit(1);
    });

    // Create empty queue
    let mut queue: Vec<QueueItem> = Vec::new();

    // Parse file
    let mut line_index = 0;
    for line in file_content.lines() {
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

        // Reject lines that have a timestamp lower than the previous line
        if let Some(previous_action) = queue.last() {
            if line_timestamp < previous_action.time {
                println!("Line {line_index}: Timestamp cannot be lower than previous action line");
                process::exit(1);
            }
        }

        // Parse actions
        let actions = parse_actions_string(line_actions_text, line_index);

        // Add actions to queue
        queue.push(QueueItem {
            time: line_timestamp,
            actions
        });
    }

    // Return populated queue
    queue
}

fn parse_actions_string(string: &str, line_index: i32) -> Vec<Action> {
    // Split into individual action strings
    let action_strings: Vec<&str> = string.split(";").collect();
    if action_strings.len() == 1 && action_strings[0].is_empty() {
        println!("Line {line_index}: Need at least one action");
        process::exit(1);
    }

    // Store actions as Vec<Action>
    let mut actions: Vec<Action> = Vec::new();
    for mut action in action_strings {
        // Ignore empty actions
        action = action.trim();
        if action.is_empty() {
            continue;
        }

        // Split into segments
        let segments: Vec<&str> = action.split_whitespace().collect();
        let action_name = segments[0];

        // Add Action to actions
        match action_name {
            "mousemove" => {
                // Validate arguments
                if segments.len() < 3 {
                    println!("Line {line_index} ({action_name}): Too few arguments! (min. 2 arguments)");
                    process::exit(1);
                }
                if segments.len() > 3 {
                    println!("Line {line_index} ({action_name}): Too many arguments provided (max. 2 arguments)");
                    process::exit(1);
                }

                // Parse X position
                let x: i32 = segments[1].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} ({action_name}): Invalid X position {:?} ({error})", segments[1]);
                    process::exit(1);
                });

                // Parse Y position
                let y: i32 = segments[2].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} ({action_name}): Invalid Y position {:?} ({error})", segments[2]);
                    process::exit(1);
                });

                // Add to actions
                actions.push(Action::MouseMove { x, y });
            }
            "mousedown" | "mouseup" => {
                // Validate arguments
                if segments.len() < 2 {
                    println!("Line {line_index} ({action_name}): No argument provided");
                    process::exit(1);
                }
                if segments.len() > 2 {
                    println!("Line {line_index} ({action_name}): Too many arguments provided (max. 1 argument)");
                    process::exit(1);
                }

                // Parse button
                let button_number: u8 = segments[1].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} ({action_name}): Invalid button {:?} ({error})", segments[1]);
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
                        println!("Line {line_index} ({action_name}): Invalid button {:?}", segments[1]);
                        process::exit(1);
                    }
                };

                // Add to actions
                match action_name {
                    "mousedown" => actions.push(Action::MouseDown { button }),
                    "mouseup" => actions.push(Action::MouseUp { button }),
                    _ => panic!("Line {line_index} ({action_name}): Reached a branch that should be impossible to reach. This is not your fault, please report a bug on GitHub if this keeps happening!")
                }
            }
            "keydown" | "keyup" => {
                // Validate arguments
                if segments.len() < 2 {
                    println!("Line {line_index} ({action_name}): No argument provided");
                    process::exit(1);
                }
                if segments.len() > 2 {
                    println!("Line {line_index} ({action_name}): Too many arguments provided (max. 1 argument)");
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
                            println!("Line {line_index} ({action_name}): Invalid key {:?} ({error})", segments[1]);
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
                                println!("Line {line_index} ({action_name}): Invalid key {:?}", segments[1]);
                                process::exit(1);
                            }
                        }
                    }
                };

                // Add to actions
                match action_name {
                    "keydown" => actions.push(Action::KeyDown { key }),
                    "keyup" => actions.push(Action::KeyUp { key }),
                    _ => panic!("Line {line_index} ({action_name}): Reached a branch that should be impossible to reach. This is not your fault, please report a bug on GitHub if this keeps happening!")
                }
            }
            _ => {
                println!("Line {line_index}: Invalid action: {action_name:?}");
                process::exit(1);
            }
        }
    }

    // Return populated actions list
    actions
}

fn execute_action(enigo: &mut Enigo, current_time: u64, action: Action, should_execute: bool, should_log: bool) {
    match action {
        Action::MouseMove { x, y } => {
            if should_execute {
                let _ = enigo.move_mouse(x, y, Coordinate::Abs);
            }

            if should_log {
                println!("At {current_time}ms: Move mouse to {x}, {y}");
            }
        }
        Action::MouseDown { button } => {
            if should_execute {
                let _ = enigo.button(button, Direction::Press);
            }

            if should_log {
                println!("At {current_time}ms: Press mouse {button:?}");
            }
        }
        Action::MouseUp { button } => {
            if should_execute {
                let _ = enigo.button(button, Direction::Release);
            }

            if should_log {
                println!("At {current_time}ms: Release mouse {button:?}");
            }
        }
        Action::KeyDown { key } => {
            if should_execute {
                if let Err(error) = enigo.key(key, Direction::Press) {
                    println!("Failed to press key {key:?}: {error}");
                }
            }

            if should_log {
                println!("At {current_time}ms: Press key {key:?}");
            }
        }
        Action::KeyUp { key } => {
            if should_execute {
                if let Err(error) = enigo.key(key, Direction::Release) {
                    println!("Failed to press key {key:?}: {error}");
                }
            }

            if should_log {
                println!("At {current_time}ms: Release key {key:?}");
            }
        }
    }
}
