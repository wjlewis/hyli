use super::file::FileInfo;
use std::cell::RefCell;
use std::fmt;

#[derive(PartialEq, Copy, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

thread_local! {
    pub static FILE_INFO: RefCell<FileInfo> = RefCell::new(FileInfo {
        path: String::from("<unspecified>"),
        text: String::from("")
    });
}
