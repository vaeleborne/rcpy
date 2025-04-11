/*****************************************
    utils.rs
-----------------
Description: Handles some helper functions

Author: Dylan Morgan
Date 4/11/2025
*****************************************/

use std::time::Instant;
use clap::ArgMatches;
use walkdir::DirEntry;
use clap::{Arg, Command};

#[derive(Debug)]
pub struct CopyStats {
    pub files: u64,
    pub dirs: u64
}

#[derive(Debug)]
pub struct CopyOptions {
    pub show_files: bool,
    pub show_dirs: bool,
    pub recursive: bool,
	pub dry_run: bool,
    pub excludes: Vec<String>,
}

//Function to help determine if an entry is excluded based on the extension it has
pub fn is_excluded(entry: &DirEntry, excludes: &[String]) -> bool {
	if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
		excludes.iter().any(|ex| ex.trim_start_matches('.').eq_ignore_ascii_case(ext))
	} else {
		false
	}
}

//Function to display the stats of a multi-file copy
pub fn display_complete(stats: CopyStats, start_time: Instant, dry_run: bool) {

	let duration = start_time.elapsed();
	if !dry_run {
		println!("\n\n--------------COPY COMPLETE--------------\n");
		println!(
			"\n{} file(s), {} directory(ies) copied.", stats.files, stats.dirs);
		println!("Duration: {:.2?}", duration);
		println!("\n-----------------------------------------\n");
	} else {
		println!("\n\n------------DRY RUN COMPLETE------------\n");
		println!(
			"\n{} file(s), {} directory(ies) would have been copied.", stats.files, stats.dirs);
		println!("Duration: {:.2?}", duration);
		println!("\n-----------------------------------------\n");
	}
}

pub fn get_arg_matches() -> ArgMatches {
    Command::new("rcpy")
		.about("A recursive copy tool written in Rust with progress bars, dry-run mode, file exclusion, and multi-threaded support.")
		.arg(Arg::new("source")
			.required(true)
			.help("Source directory"))
		.arg(Arg::new("destination")
			.required(true)
			.help("Destination directory"))
		.arg(Arg::new("single_thread")
			.short('s')
			.long("single-thread")
			.action(clap::ArgAction::SetTrue)
			.help("Copy using only one thread, will be slower!"))
		.arg(Arg::new("only_files")
			.long("only-files")
			.action(clap::ArgAction::SetTrue)
			.conflicts_with("only_dirs")
			.help("Only output file copy operations (use --verbose or -v to output file and dir operations)"))
		.arg(Arg::new("only_dirs")
			.long("only-dirs")
			.action(clap::ArgAction::SetTrue)
			.conflicts_with("only_files")
			.help("Only output directory creation (use --verbose or -v to output file and dir operations)"))
		.arg(Arg::new("verbose")
			.short('v')
			.long("verbose")
			.action(clap::ArgAction::SetTrue)
			.help("Show per-file/directory output"))
		.arg(Arg::new("dry_run")
			.long("dry-run")
			.short('d')
			.action(clap::ArgAction::SetTrue)
			.help("Simulate copy without writing any files. NOTE(acts as though verbose is set)"))
		.arg(Arg::new("exclude")
			.long("exclude")
			.action(clap::ArgAction::Append)
			.value_name("EXT")
			.help("Exclude files by extension (e.g. --exclude .psd --exclude tmp)"))
		.arg(Arg::new("no_recursive")
			.long("no-recursive")
			.action(clap::ArgAction::SetTrue)
			.help("Copy only the top-level directory contents (non-recursive)"))
		.get_matches()
}

