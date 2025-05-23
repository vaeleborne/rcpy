mod copy;
mod utils;

use std::path::PathBuf;
use std::time::Instant;
use copy::*;
use utils::*;


fn main() {
	//Getting arguments
	let matches = get_arg_matches();

	//Setting values based on arguments
	let src = PathBuf::from(matches.get_one::<String>("source").unwrap());
	let dst = PathBuf::from(matches.get_one::<String>("destination").unwrap());

	//Ensure source is not destination!
	if src == dst {
		eprint!("Error: Source and destination paths are the same!");
		std::process::exit(1);
	}

	//OPTION VARIABLES
	let verbose = matches.get_flag("verbose");
	let quiet = !verbose;
	let only_files = matches.get_flag("only_files");
	let only_dirs = matches.get_flag("only_dirs");
	let non_recursive = matches.get_flag("no_recursive");
	let single_threaded = matches.get_flag("single_thread");
	let dry_run = matches.get_flag("dry_run");

	//The excluded file extensions
	let excludes: Vec<String> = matches
		.get_many::<String>("exclude")
		.map(|vals| vals.map(String::from).collect())
		.unwrap_or_else(Vec::new);

	//Give warning if using verbose and either and or both of the only files or only dirs flags as verbose overrides them
	if verbose && (only_files || only_dirs) {
		eprintln!("Warning: --verbose overrides --only-files and --only-dirs");
	}

	let mut options = CopyOptions {
		source: src.clone(),
		show_files: !only_dirs && !quiet,
		show_dirs: !only_files && !quiet,
		recursive: !non_recursive,
		dry_run,
		excludes,
	};

	if options.dry_run && quiet {
		if !only_dirs && !only_files {
			options.show_files = true;
			options.show_dirs = true;
		} else {
			options.show_files = only_files;
			options.show_dirs = !only_files;
		}
	}

	//Start timer then start copying!
	let start_time = Instant::now();

	//Print heading
	println!("\n--------------RUSTY COPY--------------\n");
	
	if copied_single(&src, &dst, &start_time, options.dry_run) {
		return; //Then we only copied a single file good to exit
	}
	
	//Check if we are using recursion or not and tell the user
	if options.recursive {
		println!("Recursive Mode (default)\n");
	} else {
		println!("Non-Recursive Mode\n");
	}

	if options.dry_run {
		println!("Dry-run mode enabled — no files will be written.\n");
	}
	
	run_copy(single_threaded, &src, &dst, &options, start_time);

}



