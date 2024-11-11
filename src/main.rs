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

fn main() {
    let args = Arguments::parse();

    let result = std::fs::read_to_string(&args.file);

    let script = match result {
        Ok(script) => script,
        Err(error) => {
            println!("Couldn't open input file for execution: {error}");
            process::exit(1);
        }
    };

    let mut i = 0;
    for line in script.lines() {
        i += 1;
        println!("Line {i}: {line}");
    }
}
