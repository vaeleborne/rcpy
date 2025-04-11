/*****************************************
    copy.rs
-----------------
Description: Handles the logic of copying
files and directories either single or 
multithreaded. For use in the CLI program

Author: Dylan Morgan
Date 4/11/2025
*****************************************/

use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;
use rayon::prelude::*;

use crate::utils::CopyOptions;
use crate::utils::{is_excluded, CopyStats, display_complete};

pub fn copied_single(src: &Path, dst: &Path, start_time: &Instant) -> bool {
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
		 return true;
	}
    else {
        false
    }
}

pub fn copy_parallel(
        src: &Path,
        dst: &Path,
        options: &CopyOptions
    ) -> io::Result<CopyStats> {

        //Setup our walker based on whether or not we are performing a recursive copy
        let walker = if options.recursive {
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
            if options.show_dirs {
                println!("[DIR] {}", dest_path.display());
            }
            pb.inc(1);
        }
        
        //Here is where we will loop through files and use rayon to parse in parallel
        files
            .par_iter() //This runs in parallel! Thanks Rayon!
            .for_each(|entry| {
                if is_excluded(entry, &options.excludes) {
                    pb.inc(1);
                    return;
                }

                let rel_path = entry.path().strip_prefix(src).unwrap();
                let dest_path = dst.join(rel_path);
                if let Err(err) = fs::copy(entry.path(), &dest_path) {
                    eprintln!("Failed to copy {}: {}", entry.path().display(), err);
                } else if options.show_files {
                    println!("[FILE] {} -> {}", entry.path().display(), dest_path.display());
                }
                pb.inc(1);
            });

        pb.finish_with_message("Done copying.");
    
        Ok(CopyStats {
            files: files
                .iter()
                .filter(|e| !is_excluded(e, &options.excludes))
                .count() as u64,
            dirs: dirs.len() as u64,
        })
 }

 
 pub fn copy_single_threaded(
     src: &Path,
     dst: &Path,
     options: &CopyOptions
 ) -> io::Result<CopyStats> {
     
     //Walker which varies depending on if we are doing recursive copy or not
     let walker = if options.recursive {
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
             if options.show_dirs {
                 println!("[DIR] {}", dest_path.display());
             }
             stats.dirs += 1;
         } else {	//Handle cases of files
             if is_excluded(&entry, &options.excludes) {
                pb.inc(1);
                continue;
             }
             fs::copy(entry.path(), &dest_path)?;
             if options.show_files {
                 println!("[FILE] {} -> {}", entry.path().display(), dest_path.display());
             }
             stats.files += 1;
         }
         pb.inc(1);
     }
     pb.finish_with_message("Done copying.");
 
     Ok(stats)
 }

 pub fn run_copy(
    single_threaded: bool,
    src: &Path,
    dst: &Path, 
    options: &CopyOptions,
    start_time: Instant
) {
    if single_threaded {
        println!("Single Threaded Copying...\n");
		match copy_single_threaded(&src, &dst, &options) {
			Ok(stats) => {
				display_complete(stats, start_time);
			} Err(e) => {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
    } else {
        println!("Multi-Threaded Copying...\n");
		match copy_parallel(&src, &dst, options) {
			Ok(stats) => {
				display_complete(stats, start_time);
			} Err(e) => {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
    }
}