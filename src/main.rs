use clap::Parser;
use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use std::path::PathBuf;
use std::{process, thread};

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
        y: i32,
        time: u16,
        method: Coordinate
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
    },
    Text {
        text: String
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
        thread::sleep(std::time::Duration::from_millis(wait_time));

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
            println!("Line {line_index}: Incorrectly formatted line: {line:?}");
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
                if segments.len() < 4 {
                    println!("Line {line_index} ({action_name}): Too few arguments! (min. 3 arguments)");
                    process::exit(1);
                }
                if segments.len() > 5 {
                    println!("Line {line_index} ({action_name}): Too many arguments provided (max. 4 arguments)");
                    process::exit(1);
                }

                // Parse method
                let method = match segments[1] {
                    "abs" => Coordinate::Abs,
                    "rel" => Coordinate::Rel,
                    _ => {
                        println!("Line {line_index} ({action_name}): Invalid method {:?}", segments[1]);
                        process::exit(1);
                    }
                };

                // Parse X position
                let x: i32 = segments[2].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} ({action_name}): Invalid X position {:?} ({error})", segments[2]);
                    process::exit(1);
                });

                // Parse Y position
                let y: i32 = segments[3].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} ({action_name}): Invalid Y position {:?} ({error})", segments[3]);
                    process::exit(1);
                });

                // Parse time
                let time: u16 = if segments.len() > 4 {
                    segments[4].parse().unwrap_or_else(|error| {
                        println!("Line {line_index} ({action_name}): Invalid time {:?} ({error})", segments[4]);
                        process::exit(1);
                    })
                } else {
                    0
                };

                // Add to actions
                actions.push(Action::MouseMove { x, y, time, method });
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
                    _ => unreachable!("Mouse action must be mousedown or mouseup")
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
                    _ => unreachable!("Key action must be keydown or keyup")
                }
            }
            "text" => {
                // Make sure text is provided
                if segments.len() < 2 {
                    println!("Line {line_index} ({action_name}): No text provided");
                    process::exit(1);
                }

                // Add to actions
                actions.push(Action::Text { text: segments[1..].join(" ") });
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
        Action::MouseMove { x, y, time, method } => {
            if should_execute {
                if time < 2 {
                    // Normal "snappy" mouse movement
                    // Because of a bug in enigo, we can't just pass the method to the move_mouse() function
                    match method {
                        Coordinate::Abs => {
                            let _ = enigo.move_mouse(x, y, method);
                        },
                        Coordinate::Rel => {
                            // More details on why I'm doing this can be found on the relevant GitHub issue page
                            // https://github.com/enigo-rs/enigo/issues/91
                            // Basically, the relative mouse movement code uses incorrect pixel units.
                            // The workaround for this is to first get the current mouse position,
                            // calculate a new absolute position, and move the mouse there. This probably
                            // introduces some overhead, but it'll just have to be acceptable until
                            // the enigo maintainers push a fix.
                            match enigo.location() {
                                Ok(current_pos) => {
                                    // No error occurred while trying to get the location
                                    let _ = enigo.move_mouse(x + current_pos.0, y + current_pos.1, Coordinate::Abs);
                                }
                                Err(error) => {
                                    // For some reason, we got an error trying to get the mouse position
                                    println!("At {current_time}ms: Failed to move mouse: {error}");
                                }
                            }
                        }
                    }
                } else {
                    // Create a new thread for handling timing of interpolated mouse movements
                    thread::spawn(move || {
                        // Create new enigo object for this thread to avoid dealing with cross-thread objects
                        // There is probably a better way of doing this, but I'm not about to spend
                        // my entire week figuring out the best practice for this.
                        let mut enigo = Enigo::new(&Settings::default()).unwrap_or_else(|error| {
                            println!("Failed to initialize Enigo: {error}");
                            process::exit(1);
                        });

                        // Get start position
                        let start_pos = match enigo.location() {
                            Ok(pos) => pos,
                            Err(error) => {
                                // For some reason, we got an error trying to get the mouse position
                                println!("At {current_time}ms: Failed to move mouse: {error}");
                                process::exit(1);
                            }
                        };

                        // Get relative desired position regardless of movement method
                        let move_offset = match method {
                            Coordinate::Abs => (x - start_pos.0, y - start_pos.1),
                            Coordinate::Rel => (x, y)
                        };

                        // Gradually move mouse every millisecond
                        let mut last_iteration = std::time::Instant::now();
                        for iteration in 0..time {
                            // Sleep for the time needed for this cycle to be exactly one millisecond
                            // FIXME: Something is going wrong here. I have no idea why, but I guess that's a problem for future me.
                            let elapsed = last_iteration.elapsed() + std::time::Duration::from_micros(200); // Temporarily offset this by 200 microseconds to combat incorrect timings (THIS IS VERY BAD)
                            if elapsed < std::time::Duration::from_millis(1) {
                                thread::sleep(std::time::Duration::from_millis(1) - elapsed);
                            }
                            last_iteration = std::time::Instant::now();

                            // Get absolute position for this iteration
                            let x = start_pos.0 + move_offset.0 * iteration as i32 / time as i32;
                            let y = start_pos.1 + move_offset.1 * iteration as i32 / time as i32;

                            // Set mouse position
                            let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                        }
                    });
                }
            }

            if should_log {
                match method {
                    Coordinate::Abs => println!("At {current_time}ms: Move mouse to {x}, {y} over {time}ms (absolute)"),
                    Coordinate::Rel => println!("At {current_time}ms: Move mouse by {x}, {y} over {time}ms (relative)")
                }
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
                    println!("Failed to release key {key:?}: {error}");
                }
            }

            if should_log {
                println!("At {current_time}ms: Release key {key:?}");
            }
        }
        Action::Text { text } => {
            if should_execute {
                let _ = enigo.text(text.as_str());
            }

            if should_log {
                println!("At {current_time}ms: Input text {text:?}");
            }
        }
    }
}
