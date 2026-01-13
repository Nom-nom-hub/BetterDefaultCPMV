use clap::Parser;
use console::style;
use better_cp::cli::{Cli, Commands, CopyArgs};
use better_cp::copy::{FileCopier, copy_directory};
use better_cp::parallel::{ParallelFileCopier, parallel_copy_directory};
use better_cp::error::Result;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Copy(args) => {
            if let Err(e) = handle_copy(args).await {
                eprintln!("❌ {}", e.detailed_message());
                std::process::exit(1);
            }
        }
        Commands::Move(_args) => {
            eprintln!("Move operation not yet implemented");
        }
    }

    Ok(())
}

async fn handle_copy(args: CopyArgs) -> Result<()> {
    if args.source.is_empty() {
        eprintln!("Error: no source specified");
        return Ok(());
    }

    // Dry-run mode: just show preview
    if args.dry_run {
        dry_run_preview(&args).await?;
        return Ok(());
    }

    let start = Instant::now();

    if args.source.len() == 1 {
        // Single source copy (file or directory)
        let source = &args.source[0];
        
        if source.is_dir() {
            // Directory copy - use parallel if enabled
            if args.parallel > 0 {
                parallel_copy_directory(
                    source,
                    &args.destination,
                    args.parallel,
                ).await?;
            } else {
                copy_directory(
                    source,
                    &args.destination,
                    args.overwrite,
                    !args.no_verify,
                ).await?;
            }
        } else {
            // File copy - use parallel if enabled and file is large enough
            if args.parallel > 0 {
                let parallel_copier = ParallelFileCopier::new(
                    source.clone(),
                    args.destination.clone(),
                    args.parallel,
                    !args.no_verify,
                );
                parallel_copier.copy().await?;
            } else {
                let copier = FileCopier::new(
                    source.clone(),
                    args.destination.clone(),
                    args.overwrite,
                    !args.no_verify,
                    !args.no_resume && args.resume,
                    args.atomic,
                );
                copier.copy().await?;
            }
        }
    } else {
        // Multiple sources copy (to directory)
        if !args.destination.is_dir() {
            eprintln!("Error: destination must be a directory for multiple sources");
            return Ok(());
        }

        if args.parallel > 0 {
            // Parallel copy of multiple files
            let mut handles = Vec::new();

            for source in &args.source {
                let target = args.destination.join(
                    source.file_name()
                        .ok_or_else(|| better_cp::error::Error::Custom("Invalid source path".to_string()))?
                );

                let src = source.clone();
                let parallel_threads = args.parallel;
                let verify = !args.no_verify;

                let handle = tokio::spawn(async move {
                    if src.is_dir() {
                        parallel_copy_directory(&src, &target, parallel_threads).await
                    } else {
                        let copier = ParallelFileCopier::new(src, target, parallel_threads, verify);
                        copier.copy().await
                    }
                });

                handles.push(handle);
            }

            // Wait for all to complete
            for handle in handles {
                handle.await.map_err(|e| better_cp::error::Error::Custom(e.to_string()))?
                    .map_err(|e| better_cp::error::Error::Custom(format!("Copy error: {}", e)))?;
            }
        } else {
            // Sequential copy
            for source in &args.source {
                let target = args.destination.join(
                    source.file_name()
                        .ok_or_else(|| better_cp::error::Error::Custom("Invalid source path".to_string()))?
                );
                
                if source.is_dir() {
                    // Recursive directory copy
                    copy_directory(
                        source,
                        &target,
                        args.overwrite.clone(),
                        !args.no_verify,
                    ).await?;
                } else {
                    // File copy
                    let copier = FileCopier::new(
                        source.clone(),
                        target,
                        args.overwrite.clone(),
                        !args.no_verify,
                        !args.no_resume && args.resume,
                        args.atomic,
                    );
                    copier.copy().await?;
                }
            }
        }
    }

    // Show completion summary
    let duration = start.elapsed().as_secs_f64();
    if !args.quiet {
        let count = args.source.len();
        let count_str = if count == 1 { "file" } else { "files" };
        println!(
            "\n{} {} {} in {:.2}s",
            style("✓").green(),
            count,
            count_str,
            duration
        );
    }

    Ok(())
}

async fn dry_run_preview(args: &CopyArgs) -> Result<()> {
    use better_cp::prompt;
    use std::fs;
    
    if args.source.len() == 1 {
        let source = &args.source[0];
        let target = &args.destination;
        
        if source.is_dir() {
            // Directory preview
            match calculate_dir_size(source) {
                Ok((file_count, total_size)) => {
                    let target_exists = target.exists();
                    
                    println!("\n{}", style("📋 Dry Run Preview (Directory)").cyan().bold());
                    println!("  Source: {} (directory)", source.display());
                    println!("  Files: {}", file_count);
                    println!("  Total size: {}", humansize::format_size(total_size, humansize::BINARY));
                    println!("  Target: {}", target.display());
                    
                    if target_exists {
                        println!("  Action: {} (directory already exists)", style("merge/overwrite").red());
                    } else {
                        println!("  Action: {} (new directory)", style("create").green());
                    }
                }
                Err(e) => {
                    eprintln!("❌ {}", e.detailed_message());
                    return Err(e);
                }
            }
        } else if source.is_file() {
            // File preview
            match fs::metadata(source) {
                Ok(metadata) => {
                    let target_exists = target.exists();
                    prompt::preview_operation(source, target, metadata.len(), target_exists);
                }
                Err(e) => {
                    let err = better_cp::error::Error::Io(e);
                    eprintln!("❌ {}", err.detailed_message());
                    return Err(err);
                }
            }
        } else {
            eprintln!("Error: source is not a file or directory");
            return Ok(());
        }
    } else {
        // Multiple files preview
        println!("\n{}", style("📋 Dry Run Preview (Multiple Files)").cyan().bold());
        println!("  Source files: {}", args.source.len());
        println!("  Destination: {} (directory)", args.destination.display());
        
        let mut total_size = 0;
        for source in &args.source {
            if source.is_file() {
                match fs::metadata(source) {
                    Ok(m) => total_size += m.len(),
                    Err(e) => {
                        eprintln!("Warning: could not read {}: {}", source.display(), e);
                    }
                }
            }
        }
        
        println!("  Total size: {}", humansize::format_size(total_size, humansize::BINARY));
        println!("  Action: {}", style("copy all files").green());
    }
    
    println!("\n{}", style("No files were modified (--dry-run)").green());
    
    Ok(())
}

/// Calculate directory size and file count for dry-run preview
fn calculate_dir_size(path: &std::path::Path) -> Result<(usize, u64)> {
    use std::fs;
    
    let mut file_count = 0;
    let mut total_size = 0;
    
    let entries = fs::read_dir(path)
        .map_err(|e| better_cp::error::Error::Io(e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| better_cp::error::Error::Io(e))?;
        let metadata = entry.metadata()
            .map_err(|e| better_cp::error::Error::Io(e))?;
        
        if metadata.is_file() {
            file_count += 1;
            total_size += metadata.len();
        } else if metadata.is_dir() {
            let (sub_count, sub_size) = calculate_dir_size(&entry.path())?;
            file_count += sub_count;
            total_size += sub_size;
        }
    }
    
    Ok((file_count, total_size))
}
