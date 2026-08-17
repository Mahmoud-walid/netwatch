// To run this script easily without full cargo project setup:
// 1. Install rust-script: `cargo install rust-script`
// 2. Run the script: `rust-script generate_project_content.rs [path_to_project]`
//
// Alternatively, using standard rustc:
// 1. Compile: `rustc generate_project_content.rs`
// 2. Run: `./generate_project_content [path_to_project]`

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const MAX_FILE_SIZE: u64 = 1024 * 1024 * 2; // 2MB

const IGNORE_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    ".expo",
    "android",
    "dist",
    "build",
    "scripts",
    "postman-collections",
    "blueprints",
    "assets",
    "target", // Added for Rust projects
];

const IGNORE_FILES: &[&str] = &[
    "full-project-content.md",
    ".env",
    "package-lock.json",
    "eslint.config.mts",
    "tsconfig.json",
    "README.md",
    "nodemon.json",
    ".gitignore",
    ".sentryclirc",
    "Cargo.lock", // Added for Rust projects
];

fn main() -> io::Result<()> {
    // Get command line arguments safely
    let args: Vec<String> = env::args().collect();

    // Resolve base path: use last argument if provided, otherwise default to terminal's current directory
    let base_path = if args.len() > 1 {
        Path::new(args.last().unwrap()).canonicalize()?
    } else {
        env::current_dir()?.canonicalize()?
    };

    // 🎯 Dynamically get the directory of the running executable/script to make it fully reusable
    let script_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("./"));

    println!("🚀 Starting file discovery...");

    let mut all_files = Vec::new();
    get_all_files(&base_path, &mut all_files)?;

    println!("🚀 Found {} files. Starting...\n", all_files.len());

    let mut markdown_content = String::new();
    let mut processed_count = 0;

    for file in &all_files {
        processed_count += 1;

        // Show loader in terminal
        print!("\r📦 Processing files: {}", processed_count);
        io::stdout().flush()?;

        if let Ok(relative_path) = file.strip_prefix(&base_path) {
            let relative_path_str = relative_path.to_string_lossy();

            // Print current file being processed
            println!(" => 📄 {}", relative_path_str);

            let content =
                fs::read_to_string(file).unwrap_or_else(|_| "[Could not read file]".to_string());

            markdown_content.push_str(&format!(
                "{}\n```\n{}\n```\n-----\n",
                relative_path_str, content
            ));
        }
    }

    // 🎯 Dynamically build output path next to the script/executable
    let output_path = script_dir.join("full-project-content.md");
    fs::write(&output_path, markdown_content)?;

    println!("\n\n✅ Done. Processed {} files.", all_files.len());
    println!(
        "📄 Output: {:?}",
        output_path
            .canonicalize()
            .unwrap_or(output_path.to_path_buf())
    );

    Ok(())
}

fn get_all_files(dir: &Path, array_of_files: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

            if path.is_dir() {
                if !IGNORE_DIRS.contains(&file_name.as_ref()) {
                    get_all_files(&path, array_of_files)?;
                }
            } else {
                if IGNORE_FILES.contains(&file_name.as_ref()) {
                    continue;
                }
                if let Ok(metadata) = entry.metadata() {
                    if metadata.len() > MAX_FILE_SIZE {
                        continue;
                    }
                }
                array_of_files.push(path);
            }
        }
    }
    Ok(())
}
