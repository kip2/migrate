use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn main() {
    println!("Hello, world!");
}

fn create_file() -> bool {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_create_file() {
        let filename = "./test/test_file.txt";

        // cleansing test file.
        if Path::new(filename).exists() {
            let _ = fs::remove_file(filename).expect("File not Exists");
        }

        assert_eq!(true, true);

        // cleansing test file.
        if Path::new(filename).exists() {
            let _ = fs::remove_file(filename).expect("File not Exists");
        }
    }
}
