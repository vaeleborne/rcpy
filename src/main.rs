use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io;
use std::time::Instant;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use walkdir::DirEntry;
use rayon::prelude::*;

struct CopyStats {
	files: u64,
	dirs: u64,
}

fn is_excluded(entry: &DirEntry, excludes: &[String]) -> bool {
	if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
		excludes.iter().any(|ex| ex.trim_start_matches('.').eq_ignore_ascii_case(ext))
	} else {
		false
	}
}

fn copy_recursive_parallel(
	src: &Path,
	dst: &Path,
	show_files: bool,
	show_dirs: bool,
	recursive: bool,
	excludes: &[String]
) -> io::Result<CopyStats> {
	let walker = if recursive {
		WalkDir::new(src)
	} else {
		WalkDir::new(src).max_depth(1)
	};
	let entries: Vec<_> = walker.into_iter().collect::<Result<_, _>>()?;
	let pb = ProgressBar::new(entries.len() as u64);
	pb.set_style(
		ProgressStyle::default_bar()
			.template("{bar:40.cyan/blue} {pos}/{len} [{elapsed_precise}]")
			.unwrap(),
	);

	let (dirs, files): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.file_type().is_dir());
	
	for dir in &dirs {
		let rel_path = dir.path().strip_prefix(src).unwrap();
		let dest_path = dst.join(rel_path);
		fs::create_dir_all(&dest_path)?;
		if show_dirs {
			println!("[DIR] {}", dest_path.display());
		}
		pb.inc(1);
	}
	
	files
		.par_iter() //This runs in parallel! Thanks Rayon!
		.filter(|entry| !is_excluded(entry, excludes))
		.for_each(|entry| {
			let rel_path = entry.path().strip_prefix(src).unwrap();
			let dest_path = dst.join(rel_path);
			if let Err(err) = fs::copy(entry.path(), &dest_path) {
				eprintln!("Failed to copy {}: {}", entry.path().display(), err);
			} else if show_files {
				println!("[FILE] {} -> {}", entry.path().display(), dest_path.display());
			}
			pb.inc(1);
		});
	pb.finish_with_message("Done copying.");

	Ok(CopyStats {
		files: files
			.iter()
			.filter(|e| !is_excluded(e, excludes))
			.count() as u64,
		dirs: dirs.len() as u64,
	})
}

fn copy_recursive_verbose(
	src: &Path,
	dst: &Path,
	show_files: bool,
	show_dirs: bool,
	recursive: bool
) -> io::Result<CopyStats> {
	
	let walker = if recursive {
		WalkDir::new(src)
	} else {
		WalkDir::new(src).max_depth(1)
	};
	
	let entries: Vec<_> = walker.into_iter().collect::<Result<_, _>>()?;
	let pb = ProgressBar::new(entries.len() as u64);
	pb.set_style(
		ProgressStyle::default_bar()
			.template("{bar:40.cyan/blue} {pos}/{len} [{elapsed_precise}]")
			.unwrap(),
	);

	let mut stats = CopyStats { files: 0, dirs: 0 };
	
	for entry in entries {
		let rel_path = entry.path().strip_prefix(src).unwrap();
		let dest_path = dst.join(rel_path);

		if entry.file_type().is_dir() {
			fs::create_dir_all(&dest_path)?;
			if show_dirs {
				println!("[DIR] {}", dest_path.display());
			}
			stats.dirs += 1;
		} else {
			fs::copy(entry.path(), &dest_path)?;
			if show_files {
				println!("[FILE] {} -> {}", entry.path().display(), dest_path.display());
			}
			stats.files += 1;
		}
		pb.inc(1);
	}
	pb.finish_with_message("Done copying.");

	Ok(stats)
}


fn main() {
	let matches = Command::new("rcp")
		.about("Recursive copy with verbose output, progress, and summary")
		.arg(Arg::new("source")
			.required(true)
			.help("Source directory"))
		.arg(Arg::new("destination")
			.required(true)
			.help("Destination directory"))
		.arg(Arg::new("recursive")
			.short('r')
			.long("recursive")
			.help("Copy recursively (default behavior)"))
		.arg(Arg::new("single-thread")
			.short('s')
			.long("single-thread")
			.action(clap::ArgAction::SetTrue)
			.conflicts_with("exclude")
			.help("Copy using only one thread, will be slower!"))
		.arg(Arg::new("only-files")
			.long("only-files")
			.action(clap::ArgAction::SetTrue)
			.conflicts_with("only-dirs")
			.help("Only output file copy operations"))
		.arg(Arg::new("only-dirs")
			.long("only-dirs")
			.action(clap::ArgAction::SetTrue)
			.conflicts_with("only-files")
			.help("Only output directory creation"))
		.arg(Arg::new("quiet")
			.short('q')
			.long("quiet")
			.action(clap::ArgAction::SetTrue)
			.help("Supress per-file/directory output (only show progress and summary)"))
		.arg(Arg::new("exclude")
			.long("exclude")
			.action(clap::ArgAction::Append)
			.value_name("EXT")
			.help("Exclude files by extension (e.g. --exclude .psd --exclude tmp)"))
		.arg(Arg::new("no-recursive")
			.long("no-recursive")
			.action(clap::ArgAction::SetTrue)
			.help("Copy only the top-level directory contents (non-recursive)"))
		.get_matches();

	let src = PathBuf::from(matches.get_one::<String>("source").unwrap());
	let dst = PathBuf::from(matches.get_one::<String>("destination").unwrap());

	let quiet = matches.get_flag("quiet");
	let only_files = matches.get_flag("only-files");
	let only_dirs = matches.get_flag("only-dirs");

	let show_files = !only_dirs && !quiet;
	let show_dirs = !only_files && !quiet;

	let non_recursive = matches.get_flag("no-recursive");
	let recursive = !non_recursive;

	let single_threaded = matches.get_flag("single-thread");

	let excludes: Vec<String> = matches
		.get_many::<String>("exclude")
		.map(|vals| vals.map(String::from).collect())
		.unwrap_or_else(Vec::new);

	if quiet && (only_files || only_dirs) {
		eprintln!("Warning: --quiet overrides --only-files and --only-dirs");
	}
	
	let start_time = Instant::now();

	println!("\n--------------RUSTY COPY--------------\n");
	if recursive {
		println!("Recursive Mode (default)\n");
	} else {
		println!("Non-Recursive Mode\n");
	}

	if single_threaded == true {

		println!("Single Threaded Copying...\n");
		match copy_recursive_verbose(&src, &dst, show_files, show_dirs, recursive) {
			Ok(stats) => {
				display_complete(stats, start_time);
			} Err(e) => {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
	} else {
		println!("Multi-Threaded Copying...\n");
		match copy_recursive_parallel(&src, &dst, show_files, show_dirs, recursive, &excludes) {
			Ok(stats) => {
				display_complete(stats, start_time);
			} Err(e) => {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
	}

}
fn display_complete(stats: CopyStats, start_time: Instant) {

	let duration = start_time.elapsed();
	println!("\n\n--------------COPY COMPLETE--------------\n");
	println!(
		"\n{} file(s), {} directory(ies) copied.", stats.files, stats.dirs);
	println!("Duration: {:.2?}", duration);
	println!("\n-----------------------------------------\n");
}
