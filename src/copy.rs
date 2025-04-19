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
use walkdir::DirEntry;
use std::fs;

use std::io;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;
use rayon::prelude::*;

use crate::utils::CopyOptions;
use crate::utils::{is_excluded, CopyStats, display_complete};

fn finish_progress(pb: &ProgressBar) {
    pb.finish_with_message("Done copying.");
}

pub fn copied_single(src: &Path, dst: &Path, start_time: &Instant, dry_run: bool) -> bool {
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

            if dry_run {
                let duration = start_time.elapsed();
                println!("\n\n------------DRY RUN COMPLETE------------\n");
                println!("\nWould have copied: {} -> {}", src.display(), target.display());
                println!("Duration: {:.2?}", duration);
                println!("\n-----------------------------------------\n");
                return true;
            }
    
			match fs::copy(&src, &target) {
				Ok(_) =>{ 
				let duration = start_time.elapsed();
                    println!("\n\n--------------COPY COMPLETE--------------\n");
                    println!("\nCopied: {} -> {}", src.display(), target.display());
                    println!("Duration: {:.2?}", duration);
                    println!("\n-----------------------------------------\n");
                },
				Err(e) => eprintln!("Error copying file: {}", e)
			}
		} else {

            if dry_run {
                let duration = start_time.elapsed();
                println!("\n\n------------DRY RUN COMPLETE------------\n");
                println!("\nWould have copied: {} -> {}", src.display(), dst.display());
                println!("Duration: {:.2?}", duration);
                println!("\n-----------------------------------------\n");
                return true;
            }

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
            let path = dir.path().strip_prefix(src).unwrap();
            if let Err(err) = create_directories(path, dst, options, &pb) {
                eprint!("Error Copying Directory: {}", err);
            }
        }
        
        //Here is where we will loop through files and use rayon to parse in parallel
        files
            .par_iter() //This runs in parallel! Thanks Rayon!
            .for_each(|entry| {
                if is_excluded(entry, &options.excludes) {
                    pb.inc(1);
                    return;
                }
                let path = entry.path().strip_prefix(src).unwrap();
                if let Err(err) =create_files(path, dst, options, &pb) {
                    eprint!("Error Copying File: {}", err);
                }
            });

        finish_progress(&pb);
    
        Ok(get_copy_stats(files, dirs, options))
 }

 fn get_copy_stats(files: Vec<DirEntry>, dirs: Vec<DirEntry>, options: &CopyOptions) -> CopyStats {
    CopyStats {
        files: files
            .iter()
            .filter(|e| !is_excluded(e, &options.excludes))
            .count() as u64,
        dirs: dirs.len() as u64
    }
 }

 fn create_directories(path: &Path, dst: &Path, options: &CopyOptions, pb: &ProgressBar) -> Result<(), Box<dyn std::error::Error>>{
    let rel_path = path;
    let dest_path = dst.join(rel_path);
    if options.dry_run {
        println!("[DRY RUN] mkdir {}", dest_path.display());
    } else {
        //Create directories
        fs::create_dir_all(&dest_path)?;

        //Ensure directory permissions are copied
        copy_permissions(&path, &dest_path);
        if options.show_dirs {
            println!("[DIR] {}", dest_path.display());
        }
    }
    pb.inc(1);
    Ok(())
 }

 fn create_files(path: &Path, dst: &Path, options: &CopyOptions, pb: &ProgressBar)  -> Result<(), Box<dyn std::error::Error>>{
    let rel_path = path;
    let src_path = options.source.join(path); // full absolute source path
    let real_path = fs::canonicalize(&src_path)?;
    let dest_path = dst.join(rel_path);
    if options.dry_run {
        println!("[DRY RUN] {} -> {}",real_path.display(), dest_path.display());
    } else {
        //File Copy Happens Here
        if let Err(err) = fs::copy(&real_path, &dest_path) {
            eprintln!("Failed to copy {}: {}", path.display(), err); 
        } else {
           copy_permissions(&real_path, &dest_path);
            //Show output of what file gets copied if we should
            if options.show_files 
            {
                println!("[FILE] {} -> {}",real_path.display(), dest_path.display());
            }
        }   
    }
    pb.inc(1);
    Ok(())
 }

 fn copy_permissions(path: &Path, dest_path: &Path) {
    if let Ok(metadata) = fs::metadata(&path) {
        let perms = metadata.permissions(); 
        if let Err(err) = fs::set_permissions(&dest_path, perms) {
            eprintln!("Failed to write permissions for {}: {}", dest_path.display(), err);
        }
    }
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
 
             //Getting our files and directories

        
     //Setup progress bar
     let pb = ProgressBar::new(entries.len() as u64);
     pb.set_style(
         ProgressStyle::default_bar()
             .template("{bar:40.cyan/blue} {pos}/{len} [{elapsed_precise}]")
             .unwrap(),
     );

    let (dirs, files): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| e.file_type().is_dir());
     
        //Loop through all entries
    for dir in &dirs {
        let path = dir.path().strip_prefix(src).unwrap();
        if let Err(err) = create_directories(path, dst, options, &pb) {
            eprint!("Error Copying Directory: {}", err);
        }
    }
    files
        .iter()
        .for_each(|entry| {
            if is_excluded(entry, &options.excludes) {
                pb.inc(1);
                return;
            }
            let path = entry.path().strip_prefix(src).unwrap();
            if let Err(err) = create_files(path, dst, options, &pb) {
                eprint!("Error Copying File: {}", err);
            }
        });
    finish_progress(&pb);
 
     Ok(get_copy_stats(files, dirs, options))
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
				display_complete(stats, start_time, options.dry_run);
			} Err(e) => {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
    } else {
        println!("Multi-Threaded Copying...\n");
		match copy_parallel(&src, &dst, options) {
			Ok(stats) => {
				display_complete(stats, start_time, options.dry_run);
			} Err(e) => {
				eprintln!("Error: {}", e);
				std::process::exit(1);
			}
		}
    }
}