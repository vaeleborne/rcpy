use clap::{Arg, Command};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io;
use std::time::Instant;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use walkdir::DirEntry;
use rayon::prelude::*;

//Struct to hold information about how many files and directories are copied!
struct CopyStats {
	files: u64,
	dirs: u64,
}

//Function to help determine if an entry is excluded based on the extension it has
fn is_excluded(entry: &DirEntry, excludes: &[String]) -> bool {
	if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
		excludes.iter().any(|ex| ex.trim_start_matches('.').eq_ignore_ascii_case(ext))
	} else {
		false
	}
}

//Main copy function, uses multithreading with rayon!
fn copy_parallel(
	src: &Path,
	dst: &Path,
	show_files: bool,
	show_dirs: bool,
	recursive: bool,
	excludes: &[String]
) -> io::Result<CopyStats> {
	//Setup our walker based on whether or not we are performing a recursive copy
	let walker = if recursive {
		WalkDir::new(src)
	} else {
		WalkDir::new(src).max_depth(1)
	};

	//Get entries via our walker
	let entries: Vec<_> = walker.into_iter().collect::<Result<_, _>>()?;

	//Setting up our progress bar
	let pb = ProgressBar::new(entries.len() as u64);
	pb.set_style(
		ProgressStyle::default_bar()
			.template("{bar:40.cyan/blue} {pos}/{len} [{elapsed_precise}]")
			.unwrap(),
	);

	//Getting our files and directories
	let (dirs, files): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.file_type().is_dir());
	
	//Loop through directories
	for dir in &dirs {
		let rel_path = dir.path().strip_prefix(src).unwrap();
		let dest_path = dst.join(rel_path);
		fs::create_dir_all(&dest_path)?;
		if show_dirs {
			println!("[DIR] {}", dest_path.display());
		}
		pb.inc(1);
	}
	
	//Here is where we will loop through files and use rayon to parse in parallel
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

//Basic Copy Single Threaded
fn copy_single_threaded(
	src: &Path,
	dst: &Path,
	show_files: bool,
	show_dirs: bool,
	recursive: bool
) -> io::Result<CopyStats> {
	
	//Walker which varies depending on if we are doing recursive copy or not
	let walker = if recursive {
		WalkDir::new(src)
	} else {
		WalkDir::new(src).max_depth(1)
	};
	
	//Get entries
	let entries: Vec<_> = walker.into_iter().collect::<Result<_, _>>()?;

	//Setup progress bar
	let pb = ProgressBar::new(entries.len() as u64);
	pb.set_style(
		ProgressStyle::default_bar()
			.template("{bar:40.cyan/blue} {pos}/{len} [{elapsed_precise}]")
			.unwrap(),
	);

	//Initialize our stats
	let mut stats = CopyStats { files: 0, dirs: 0 };
	
	//Loop through all entries
	for entry in entries {
		let rel_path = entry.path().strip_prefix(src).unwrap();
		let dest_path = dst.join(rel_path);

		//Handle cases of directories
		if entry.file_type().is_dir() {
			fs::create_dir_all(&dest_path)?;
			if show_dirs {
				println!("[DIR] {}", dest_path.display());
			}
			stats.dirs += 1;
		} else {	//Handle cases of files
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
	//Getting arguments
	let matches = Command::new("rcpy")
		.about("A Rust based recursive copy with a progress bar and summary")
		.arg(Arg::new("source")
			.required(true)
			.help("Source directory"))
		.arg(Arg::new("destination")
			.required(true)
			.help("Destination directory"))
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
			.help("Only output file copy operations (use --verbose or -v to output file and dir operations)"))
		.arg(Arg::new("only-dirs")
			.long("only-dirs")
			.action(clap::ArgAction::SetTrue)
			.conflicts_with("only-files")
			.help("Only output directory creation (use --verbose or -v to output file and dir operations)"))
		.arg(Arg::new("verbose")
			.short('v')
			.long("verbose")
			.action(clap::ArgAction::SetTrue)
			.help("Show per-file/directory output"))
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

	//Setting values based on arguments
	let src = PathBuf::from(matches.get_one::<String>("source").unwrap());
	let dst = PathBuf::from(matches.get_one::<String>("destination").unwrap());
	
	let verbose = matches.get_flag("verbose");
	let quiet = !verbose;
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

	if verbose && (only_files || only_dirs) {
		eprintln!("Warning: --verbose overrides --only-files and --only-dirs");
	}
	
	//Start timer then start copying!
	let start_time = Instant::now();

	println!("\n--------------RUSTY COPY--------------\n");
	
	//Getting metadata so we can check if we are copying a single file
	let metadata = match std::fs::metadata(&src) {
		Ok(m) => m,
		Err(e) => {
			eprintln!("Error reading source: {}", e);
			std::process::exit(1);

		}
	};

	//Handle case of copying a single file!
	if metadata.is_file() {
		if dst.is_dir() {
			//If destination is a folder, append filename
			let filename = src.file_name().unwrap();
			let target = dst.join(filename);
			match fs::copy(&src, &target) {
				Ok(_) =>{ 
				let duration = start_time.elapsed();
				println!("\n\n--------------COPY COMPLETE--------------\n");
				println!("\nCopied: {} -> {}", src.display(), target.display());
				println!("Duration: {:.2?}", duration);
				println!("\n-----------------------------------------\n");},
				Err(e) => eprintln!("Error copying file: {}", e)
			}
		} else {
			match fs::copy(&src, &dst) {
				Ok(_) =>{
					let duration = start_time.elapsed();
					println!("\n\n--------------COPY COMPLETE--------------\n");
					println!("Copied: {} -> {}", src.display(), dst.display());
					println!("Duration: {:.2?}", duration);
					println!("\n-----------------------------------------\n");
				},
				Err(e) => eprintln!("Error copying file: {}", e)

			}
		}
		return;
	}
	
	//Check if we are using recursion or not and tell the user
	if recursive {
		println!("Recursive Mode (default)\n");
	} else {
		println!("Non-Recursive Mode\n");
	}
	
	//Case of running single threaded copy
	if single_threaded == true {

		println!("Single Threaded Copying...\n");
		match copy_single_threaded(&src, &dst, show_files, show_dirs, recursive) {
			Ok(stats) => {
				display_complete(stats, start_time);
			} Err(e) => {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
	} else { //Running multi-threaded copy
		println!("Multi-Threaded Copying...\n");
		match copy_parallel(&src, &dst, show_files, show_dirs, recursive, &excludes) {
			Ok(stats) => {
				display_complete(stats, start_time);
			} Err(e) => {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
	}

}

//Function to display the stats of a multi-file copy
fn display_complete(stats: CopyStats, start_time: Instant) {

	let duration = start_time.elapsed();
	println!("\n\n--------------COPY COMPLETE--------------\n");
	println!(
		"\n{} file(s), {} directory(ies) copied.", stats.files, stats.dirs);
	println!("Duration: {:.2?}", duration);
	println!("\n-----------------------------------------\n");
}

