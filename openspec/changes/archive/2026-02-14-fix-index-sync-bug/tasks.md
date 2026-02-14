## 1. Add Database.clear_all() method

- [x] 1.1 Add clear_all() method to Database struct in src/db/mod.rs
- [x] 1.2 Implement DELETE statements for files, chunks, and symbols tables
- [x] 1.3 Test clear_all() method independently

## 2. Fix clear_index() in engine.rs

- [x] 2.1 Update clear_index() to call self.db.clear_all()
- [x] 2.2 Remove the broken Database replacement code
- [x] 2.3 Ensure proper error handling

## 3. Verify the fix

- [x] 3.1 Test flashgrep clear command
- [x] 3.2 Test flashgrep index after clearing
- [x] 3.3 Verify text search returns results
- [x] 3.4 Verify symbol search returns results
- [x] 3.5 Confirm both searches are consistent

## 4. Code quality

- [x] 4.1 Add logging for database clear operation
- [x] 4.2 Run clippy and fix any warnings
- [x] 4.3 Verify no compilation errors
