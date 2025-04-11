use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

struct CopyStats {
	files: u64,
	dirs: u64,
}

fn copy_recursive_verbose(
	src: &Path,
	dst: &Path,
	show_files: bool,
	show_dirs: bool
) -> io::Result<CopyStats> {
	
	let entries: Vec<_> = WalkDir::new(src).into_iter().collect::<Result<_, _>>()?;
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
		.get_matches();

	let src = PathBuf::from(matches.get_one::<String>("source").unwrap());
	let dst = PathBuf::from(matches.get_one::<String>("destination").unwrap());

	let only_files = matches.get_flag("only-files");
	let only_dirs = matches.get_flag("only-dirs");

	let show_files = !only_dirs;
	let show_dirs = !only_files;

	match copy_recursive_verbose(&src, &dst, show_files, show_dirs) {
		Ok(stats) => {
			println!(
				"\nCopy complete: {} file(s), {} directorie(s) copied.", stats.files, stats.dirs);
		} Err(e) => {
			eprintln!("Error: {}", e);
			std::process::exit(1);
		}
	}
}
