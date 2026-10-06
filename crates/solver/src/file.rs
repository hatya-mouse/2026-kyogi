use std::path::Path;

pub fn read_from_relative_path(path: &Path) -> std::io::Result<String> {
    // Get an absolute path to the file
    let absolute_path = path.canonicalize()?;
    // Get the content of the file
    std::fs::read_to_string(absolute_path)
}
