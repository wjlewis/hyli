use super::common::FILE_INFO;
use std::fs;

pub struct FileInfo {
    pub path: String,
    pub text: String,
}

pub fn read_file(path: &str) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let text = fs::read_to_string(path)?;

    FILE_INFO.with(|info| {
        let mut info = info.borrow_mut();
        info.path = String::from(path);
        info.text = text;
    });

    Ok(())
}
