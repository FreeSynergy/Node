// fsn-store-validator
//
// Standalone store catalog validator.
// No private FreeSynergy.Lib dependencies — safe to build in CI.
//
// Usage:
//   fsn-store-validator <store-dir> <namespace>
//
// Example:
//   fsn-store-validator ./Store node

use std::{env, path::Path, process};

mod validate_store;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: fsn-store-validator <store-dir> <namespace>");
        process::exit(2);
    }
    let store_dir = Path::new(&args[1]);
    let namespace = &args[2];

    if let Err(e) = validate_store::run(store_dir, namespace) {
        eprintln!("Error: {e:?}");
        process::exit(1);
    }
}
