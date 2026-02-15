pub mod engine;
pub mod initial_scanner;
pub mod scanner;
pub mod state;

pub use engine::Indexer;
pub use initial_scanner::{InitialScanner, ScanResult};
pub use scanner::{
    is_binary_file, is_oversized_file, should_ignore_directory, should_index_file, FileScanner,
    FlashgrepIgnore,
};
pub use state::{FileMetadata, IndexState, ThreadSafeIndexState};
