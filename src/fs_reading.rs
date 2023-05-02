use std::fs;

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
