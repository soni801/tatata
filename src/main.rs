use std::path::PathBuf;
use std::process;
use clap::Parser;

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
    time: u32,
    actions: Vec<Action>
}

#[derive(Debug)]
enum Action {
    MouseMoveAction {
        x: i16,
        y: i16,
        time: u16
    },
    MousePressAction {
        button: u8
    },
    KeyPressAction {
        key: char
    }
}

fn main() {
    // Vec of action queue
    let mut queue: Vec<QueueItem> = Vec::new();

    // Get file
    let args = Arguments::parse();
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
        let line_timestamp: u32 = line_timestamp_text.parse().unwrap_or_else(|error| {
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
                    println!("Line {line_index} (mousemove): Need at least 2 arguments");
                    process::exit(1);
                }
                if segments.len() > 4 {
                    println!("Line {line_index} (mousemove): Too many arguments provided (max 3 arguments)");
                    process::exit(1);
                }

                // Parse X position
                let x: i16 = segments[1].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} (mousemove): Invalid X position {:?} ({error})", segments[1]);
                    process::exit(1);
                });

                // Parse Y position
                let y: i16 = segments[2].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} (mousemove): Invalid Y position {:?} ({error})", segments[2]);
                    process::exit(1);
                });

                // Parse time
                let time: u16;
                if segments.len() == 4 {
                    time = segments[3].parse().unwrap_or_else(|error| {
                        println!("Line {line_index} (mousemove): Invalid time {:?} ({error})", segments[3]);
                        process::exit(1);
                    });
                } else {
                    time = 0;
                }

                // Add to actions
                actions.push(Action::MouseMoveAction { x, y, time });
            } else if action.starts_with("mousepress") {
                // Validate arguments
                let segments: Vec<&str> = action.split_whitespace().collect();
                if segments.len() < 2 {
                    println!("Line {line_index} (mousepress): No argument provided");
                    process::exit(1);
                }
                if segments.len() > 2 {
                    println!("Line {line_index} (mousepress): Too many arguments provided (max 1 argument)");
                    process::exit(1);
                }

                // Parse button
                let button: u8 = segments[1].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} (mousepress): Invalid button {:?} ({error})", segments[1]);
                    process::exit(1);
                });

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
                    println!("Line {line_index} (keypress): Too many arguments provided (max 1 argument)");
                    process::exit(1);
                }

                // Parse key
                let key: char = segments[1].parse().unwrap_or_else(|error| {
                    println!("Line {line_index} (keypress): Invalid key {:?} ({error})", segments[1]);
                    process::exit(1);
                });

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

    println!("{:?}", queue);
}
