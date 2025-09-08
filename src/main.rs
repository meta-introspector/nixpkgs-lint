use std::{env::current_dir, fs::read_to_string, path::PathBuf, process::ExitCode};

use clap::{crate_version, Parser};
use display::{print_matches, DisplayFormats};
use find::find_nix_files;
use find_lints::find_lints;
use indicatif::{ParallelProgressIterator, ProgressBar};
use queries::{add_default_queries, add_unfinished_queries};
use query::{AMatch, AQuery};
use rayon::prelude::*;

use log::{debug, info, warn, error}; // Import log macros

mod display;
mod find;
mod find_lints;
mod queries;
mod query;

fn main() -> ExitCode {
    // Initialize the logger
    env_logger::init();
    info!("nixpkgs-lint: Starting process.");

    let args = Opt::parse();
    debug!("nixpkgs-lint: Parsed arguments: {:?}", args);

    let mut match_vec: Vec<AMatch> = Vec::new();

    let mut queries: Vec<AQuery> = Vec::new();

    info!("nixpkgs-lint: Adding default queries.");
    add_default_queries(&mut queries);

    if args.include_unfinished_lints {
        info!("nixpkgs-lint: Including unfinished lints.");
        add_unfinished_queries(&mut queries);
    };
    debug!("nixpkgs-lint: Total queries loaded: {}", queries.len());

    for mut path in args.file {
        info!("nixpkgs-lint: Processing path: {}", path.to_string_lossy());
        if let Ok(false) = &path.try_exists() {
            error!("nixpkgs-lint: path '{}' does not exist", path.to_string_lossy());
            return ExitCode::FAILURE;
        }
        if path.to_string_lossy() == "." {
            path = current_dir().unwrap();
            debug!("nixpkgs-lint: Resolved '.' to current directory: {}", path.to_string_lossy());
        }
        
        info!("nixpkgs-lint: Finding Nix files in: {}", path.to_string_lossy());
        let entries = find_nix_files(&path);
        info!("nixpkgs-lint: Found {} Nix files.", entries.len());
        let length: u64 = entries.len().try_into().unwrap();
        let mut pb = ProgressBar::hidden();
        if length > 1000 {
            pb = ProgressBar::new(length);
            info!("nixpkgs-lint: Initializing progress bar for {} files.", length);
        }

        match_vec.par_extend(entries.into_par_iter().progress_with(pb).flat_map(|entry| {
            debug!("nixpkgs-lint: Processing file: {:?}", entry);
            let file_contents = read_to_string(&entry).unwrap(); // Consider adding error handling here
            debug!("nixpkgs-lint: Read contents of file: {:?}\n", entry);

            find_lints(
                &entry,
                file_contents.trim(),
                &queries,
                &args.node_debug,
                args.timeout_micros, // Pass timeout_micros
            )
        }));
        info!("nixpkgs-lint: Finished processing files for path: {}", path.to_string_lossy());
    }

    if !match_vec.is_empty() {
        info!("nixpkgs-lint: Found {} lint matches.", match_vec.len());
        print_matches(&args.format, &match_vec);
        info!("nixpkgs-lint: Exiting with failure due to lint matches.");
        return ExitCode::FAILURE;
    }

    info!("nixpkgs-lint: No lint matches found. Exiting with success.");
    ExitCode::SUCCESS
}

#[derive(Parser, Debug)]
#[clap(version = crate_version!())]
struct Opt {
    /// Files or directories
    #[clap(value_name = "FILES/DIRECTORIES")]
    file: Vec<PathBuf>,

    /// Output format
    #[clap(value_enum, long, default_value_t = DisplayFormats::Ariadne)]
    format: DisplayFormats,

    /// debug nodes
    #[clap(long = "node-debug")]
    node_debug: bool,

    /// use lints which haven't been fixed in nixpkgs yet
    #[clap(long = "include-unfinished-lints")]
    include_unfinished_lints: bool,

    /// enable if running in nixpkgs ci
    #[clap(
        conflicts_with = "include_unfinished_lints",
        long = "running-in-nixpkgs-ci"
    )]
    running_in_nixpkgs_ci: bool,

    /// Timeout for tree-sitter parsing in microseconds
    #[clap(long, default_value_t = 0)] // 0 means no timeout
    timeout_micros: u64,
}
