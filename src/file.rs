use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::{self, Write};
use std::path::Path;

pub fn create_file(filepath: &str, contents: &str) -> io::Result<()> {
    let mut file = File::create(filepath)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub fn read_file(filepath: &str) -> io::Result<String> {
    let mut file = File::open(filepath)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

fn create_migration_file() -> bool {
    // Create Migrations directory if it does not exist
    let dir_path = Path::new("./Migrations");
    if !dir_path.exists() {
        fs::create_dir_all(dir_path).expect("test");
    }

    // create empty sql up file
    let filepath = "./Migrations/up.sql";
    let _ = create_file(filepath, "");

    // create empty sql down file
    let filepath = "./Migrations/down.sql";
    let _ = create_file(filepath, "");

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_create_migration_file() {
        assert_eq!(create_migration_file(), true);
    }

    #[test]
    fn test_read_file() {
        let filepath = "./test/test_read_file.txt";

        // cleansing test file.
        if Path::new(filepath).exists() {
            let _ = fs::remove_file(filepath).expect("File not Exists");
        }

        // create test file.
        let contents = "Read test";
        let _ = create_file(filepath, contents);

        assert_eq!(contents, read_file(filepath).unwrap());

        // cleansing test file.
        if Path::new(filepath).exists() {
            let _ = fs::remove_file(filepath).expect("File not Exists");
        }

        // create empty file.
        let contents = "Read test\nHell Word!";
        let _ = create_file(filepath, contents);

        assert_eq!(contents, read_file(filepath).unwrap());

        // cleansing test file.
        if Path::new(filepath).exists() {
            let _ = fs::remove_file(filepath).expect("File not Exists");
        }
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
        let contents = "Create Test!";
        assert!(create_file(filepath, contents).is_ok());

        // test file is not empty.
        let metadata = fs::metadata(filepath).unwrap();
        assert_ne!(metadata.len(), 0);

        // test file contents.
        assert_eq!(read_file(filepath).unwrap(), contents);

        // cleansing test file.
        if Path::new(filepath).exists() {
            let _ = fs::remove_file(filepath).expect("File not Exists");
        }
    }
}
