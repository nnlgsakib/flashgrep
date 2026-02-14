## Why

Flashgrep has a critical bug where the text search index and metadata database get out of sync. After running `flashgrep clear`, the text index is cleared but the metadata database still contains file records. This causes subsequent `flashgrep index` commands to skip all files (thinking they're already indexed), resulting in an empty text search index while symbol search still works from the stale database.

## What Changes

- **Fix clear_index method**: Actually clear the SQLite database tables, not just the Tantivy index
- **Add Database.clear_all() method**: New method to delete all records from files, chunks, and symbols tables
- **Update engine.rs**: Use the new clear_all() method in clear_index()
- **Fix needs_reindex logic**: Ensure it properly detects when text index is missing

## Capabilities

### New Capabilities
- None

### Modified Capabilities
- `indexing-engine`: Fix index clearing to properly synchronize text and metadata stores

## Impact

- **Modified Files**: 
  - `src/db/mod.rs` - Add clear_all() method
  - `src/index/engine.rs` - Fix clear_index() to use clear_all()
- **No Breaking Changes**: Bug fix only
- **User Experience**: Users can now properly clear and rebuild indexes
- **Data Integrity**: Ensures text search and symbol search use consistent data
