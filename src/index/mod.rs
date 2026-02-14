pub mod engine;
pub mod scanner;

pub use engine::Indexer;
pub use scanner::{
    is_binary_file, is_oversized_file, should_ignore_directory, should_index_file, FileScanner,
    FlashgrepIgnore,
};
