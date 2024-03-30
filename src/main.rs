use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    println!("Hello, world!");
}

fn create_file(filepath: &str, contents: &str) -> io::Result<()> {
    let mut file = File::create(filepath)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Read;

    #[test]
    fn test_read_file() {
        assert_eq!(true, true);
    }

    #[test]
    fn test_create_file() {
        let filepath = "./test/test_file.txt";

        // cleansing test file.
        if Path::new(filepath).exists() {
            let _ = fs::remove_file(filepath).expect("File not Exists");
        }

        // test result to create file
        assert!(create_file(filepath, "").is_ok());

        // test create empty file.
        let metadata = fs::metadata(filepath).unwrap();
        assert_eq!(metadata.len(), 0);

        // cleansing test file.
        if Path::new(filepath).exists() {
            let _ = fs::remove_file(filepath).expect("File not Exists");
        }

        // test create
        // test result to create file
        assert!(create_file(filepath, "Create Test!").is_ok());

        // test create empty file.
        let metadata = fs::metadata(filepath).unwrap();
        assert_ne!(metadata.len(), 0);

        // cleansing test file.
        if Path::new(filepath).exists() {
            let _ = fs::remove_file(filepath).expect("File not Exists");
        }
    }
}
