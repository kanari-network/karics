use std::fs::File;
use std::io::{self, Read};

/// Reads the contents of a file into a string
///
/// # Arguments
/// * `path` - Path to the file to read
///
/// # Returns
/// * `io::Result<String>` - The file contents or an error
pub fn read_file(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_read_file() {
        // Create a temporary test file
        let test_content = "test content";
        fs::write("test.txt", test_content).unwrap();
        
        // Read the file
        let result = read_file("test.txt").unwrap();
        assert_eq!(result, test_content);
        
        // Clean up
        fs::remove_file("test.txt").unwrap();
    }

    #[test]
    fn test_read_nonexistent_file() {
        let result = read_file("nonexistent.txt");
        assert!(result.is_err());
    }
}



