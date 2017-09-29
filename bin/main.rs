//! # iarmapcmp
//!
//! The command-line program compares the module summary sections of two IAR map
//! files.
//!
//! ## Usage
//!
//! ```text
//! iarmapcmp.exe [left-map-file] [right-map-file]
//! ```

extern crate iarmap;

mod analytics;
use analytics::analyze;

use std::env;
use std::fs::File;
use iarmap::parse_map_file;

/// Handle command-line arguments
fn handle_args(args: Vec<String>) -> Result<(String, String), &'static str> {
    if args.len() != 3 {
        Result::Err("Please provide the paths for two IAR map files")
    } else {
        Ok((args[1].clone(), args[2].clone()))
    }
}

fn main() {

    let args = handle_args(env::args().collect());
    if let &Err(ref msg) = &args {
        println!("Error: {}", msg);
        std::process::exit(1);
    }

    let (left, right) = args.unwrap();

    let (left, right) = match (File::open(left), File::open(right)) {
        (Ok(left), Ok(right)) => (left, right),
        (Err(msg), _) | (_, Err(msg)) => {
            println!("Error: {}", msg);
            std::process::exit(1);
        }
    };

    let left = parse_map_file(left);
    let right = parse_map_file(right);

    if let &Err(ref msg) = &left {
        println!("Error on left file: {}", msg);
        std::process::exit(1);
    }

    if let &Err(ref msg) = &right {
        println!("Error on right file: {}", msg);
        std::process::exit(1);
    }

    analyze(left.unwrap(), right.unwrap());
}
