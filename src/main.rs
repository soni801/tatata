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
    MouseMoveAction,
    MousePressAction,
    KeyPressAction
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
            println!("Line {line_index}: Incorrectly formatted timestamp: \"{line_timestamp_text}\" ({error})");
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
                actions.push(Action::MouseMoveAction);
            } else if action.starts_with("mousepress") {
                actions.push(Action::MousePressAction);
            } else if action.starts_with("keypress") {
                actions.push(Action::KeyPressAction);
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
