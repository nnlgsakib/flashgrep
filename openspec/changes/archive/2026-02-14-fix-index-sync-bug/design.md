## Context

The flashgrep indexing system uses two storage mechanisms:
1. **Tantivy text index** (`text_index/`) - For full-text search
2. **SQLite metadata database** (`metadata.db`) - For file metadata, chunks, and symbols

These two stores must remain synchronized. When a user runs `flashgrep clear`, both stores should be cleared. However, the current implementation only clears the Tantivy index, leaving the SQLite database intact.

## Root Cause

In `src/index/engine.rs`, the `clear_index()` method:
```rust
pub fn clear_index(&mut self) -> FlashgrepResult<()> {
    info!("Clearing index...");
    
    // Clear Tantivy index
    self.writer.delete_all_documents()?;
    self.writer.commit()?;
    
    // Clear database (recreate it)
    drop(std::mem::replace(
        &mut self.db,
        Database::open(&self.paths.metadata_db())?,
    ));
    
    info!("Index cleared");
    Ok(())
}
```

The comment says "Clear database (recreate it)" but the code only drops and replaces the Database connection handle. It does NOT actually delete any data from the SQLite file. The `Database::open()` just opens a connection to the existing database file with all its data.

## Solution

Add a `clear_all()` method to the Database struct that executes DELETE statements on all tables, then use it in `clear_index()`.

## Implementation Plan

1. Add `clear_all()` method to `Database` in `src/db/mod.rs`
2. Update `clear_index()` in `src/index/engine.rs` to call `self.db.clear_all()`
3. Ensure foreign key constraints are handled properly (ON DELETE CASCADE is already set up)

## Testing Strategy

1. Index a repository
2. Run `flashgrep clear`
3. Run `flashgrep index` again
4. Verify text search returns results
5. Verify both text and symbol searches return consistent results

## Risks

- **Data Loss**: The clear command will now actually delete data (which is the intended behavior)
- **Performance**: DELETE operations on large databases might be slow (acceptable for clear operation)
