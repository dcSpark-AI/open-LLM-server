use std::fs;
use std::path::Path;

/// Finds the first `.bin` file in the current directory and returns its path.
/// If no `.bin` files are found, returns `None`.
pub fn find_local_model() -> Option<String> {
    // Get a list of files in the current directory
    let dir = ".";
    let files = match fs::read_dir(dir) {
        Ok(files) => files,
        Err(_) => return None,
    };
    // Iterate over the files and return the first `.bin` file found
    for file in files {
        let path = match file {
            Ok(file) => file.path(),
            Err(_) => continue,
        };
        let extension = match path.extension() {
            Some(extension) => extension.to_string_lossy(),
            None => continue,
        };
        if extension == "bin" {
            return Some(path.to_str().unwrap().to_string());
        }
    }
    None
}

// Closes the app if the model file does not exist
pub fn model_file_close_check(model_path: &str) {
    if !(Path::new(model_path).exists() && Path::new(model_path).is_file()) {
        println!("Error: Model file could not be found/read, unable to start.");
        println!("Please ensure you have a .bin in the same folder as this executable,");
        println!("or that you specify the correct path via the '-p' option.");
        std::process::exit(1);
    }
}
