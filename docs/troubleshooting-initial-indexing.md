# Troubleshooting Initial Indexing

This guide helps you resolve common issues with the initial indexing feature in flashgrep.

## Common Issues

### Issue: Initial Scan is Very Slow

**Symptoms:**
- Progress logs show very low "files per second" rate
- Scan takes many minutes or hours for moderate-sized repositories
- High CPU or disk usage during scan

**Solutions:**

1. **Update ignore patterns**
   ```bash
   # Add common large directories to .flashgrepignore
   echo -e "node_modules/\ntarget/\ndist/\nbuild/\n.git/" >> .flashgrepignore
   ```

2. **Reduce the scope of indexed files**
   Edit `.flashgrep/config.json`:
   ```json
   {
     "extensions": ["rs", "js", "ts", "py"],
     "ignored_dirs": [".git", "node_modules", "target", "dist", "build", "vendor"]
   }
   ```

3. **Check for circular symlinks**
   ```bash
   # Find circular symlinks
   find . -type l -exec test ! -e {} \; -print
   ```

4. **Monitor disk I/O**
   ```bash
   # On Linux/Mac, check if disk is the bottleneck
   iostat -x 1
   ```

### Issue: "Permission Denied" Errors During Scan

**Symptoms:**
- Logs show errors like "Failed to process /path/to/file: Permission denied"
- Some files are not being indexed

**Solutions:**

1. **Check file permissions**
   ```bash
   # Find files without read permission
   find . -type f ! -readable
   
   # Fix permissions (be careful!)
   chmod -R +r .
   ```

2. **Add restricted directories to ignore patterns**
   ```bash
   echo "restricted_directory/" >> .flashgrepignore
   ```

3. **Run with appropriate permissions**
   ```bash
   # If you need to index system files, run as root (not recommended)
   sudo flashgrep start
   ```

### Issue: Index State File Corruption

**Symptoms:**
- Error: "Invalid index state" on startup
- Index state file is empty or unreadable
- Watcher fails to start

**Solutions:**

1. **Remove corrupted index state**
   ```bash
   rm .flashgrep/index-state.json
   # Or your configured index_state_path
   ```

2. **The watcher will recreate it** on next startup with a fresh empty state

3. **Check disk space**
   ```bash
   df -h
   ```

### Issue: Changes Not Detected After Restart

**Symptoms:**
- Files modified while watcher was offline are not indexed
- New files created during downtime don't appear in search results

**Solutions:**

1. **Verify initial indexing is enabled**
   ```bash
   cat .flashgrep/config.json | grep enable_initial_index
   # Should show: "enable_initial_index": true
   ```

2. **Check scan results in logs**
   Look for lines like:
   ```
   INFO Initial scan complete: X scanned, Y added, Z modified, W deleted
   ```

3. **Force a re-scan**
   ```bash
   # Stop and restart the watcher
   flashgrep stop
   rm .flashgrep/index-state.json
   flashgrep start
   ```

### Issue: High Memory Usage During Scan

**Symptoms:**
- System runs out of memory during initial scan
- Large repositories cause OOM (Out of Memory) errors
- Swap usage spikes

**Solutions:**

1. **Reduce parallel processing**
   The scanner processes files sequentially, but you can limit other system resources:
   ```bash
   # Limit CPU usage (Linux)
   cpulimit -p $(pgrep flashgrep) -l 50
   ```

2. **Exclude large file types**
   ```bash
   # Add to .flashgrepignore
   echo -e "*.bin\n*.dat\n*.zip\n*.tar.gz" >> .flashgrepignore
   ```

3. **Split large repositories**
   Consider using multiple flashgrep instances for different parts of a monorepo

### Issue: Broken Symlinks Cause Errors

**Symptoms:**
- Warnings about broken symlinks in logs
- Scan slows down due to symlink resolution attempts

**Solutions:**

1. **Ignore symlinked directories**
   ```bash
   # Find all symlinks
   find . -type l
   
   # Add symlink targets to .flashgrepignore
   ```

2. **Remove broken symlinks**
   ```bash
   find . -type l ! -exec test -e {} \; -delete
   ```

### Issue: Index State Not Updating

**Symptoms:**
- Files are scanned but index-state.json doesn't update
- Changes are lost after watcher restart

**Solutions:**

1. **Check file system permissions**
   ```bash
   ls -la .flashgrep/
   # Ensure you have write permission
   ```

2. **Verify disk space**
   ```bash
   df -h .flashgrep/
   ```

3. **Check for file locking issues**
   ```bash
   # On Windows, check for locked files
   # On Linux/Mac:
   lsof .flashgrep/index-state.json
   ```

## Debugging Tips

### Enable Debug Logging

```bash
RUST_LOG=debug flashgrep start
```

This will show detailed information about:
- Each file being scanned
- Ignore pattern matches
- Index state updates
- Error details

### Check Index State Contents

```bash
# View the index state (JSON format)
cat .flashgrep/index-state.json | jq '.'

# Count indexed files
python3 -c "import json; d=json.load(open('.flashgrep/index-state.json')); print(len(d.get('files', {})))"
```

### Monitor Scan Progress

Watch the logs for progress updates:
```bash
# In another terminal while watcher is running
tail -f .flashgrep/logs/flashgrep.log
```

Look for:
- `Initial indexing progress: X files scanned`
- `Initial scan complete: ...`

### Verify Ignore Patterns

Test if a file should be ignored:
```bash
# Check if pattern matches
grep "pattern" .flashgrepignore
```

### Check File Counts

Compare actual files vs indexed files:
```bash
# Count actual files (excluding ignored directories)
find . -type f \
  -not -path "./.flashgrep/*" \
  -not -path "./.git/*" \
  -not -path "./node_modules/*" \
  -not -path "./target/*" | wc -l

# Count indexed files
python3 -c "import json; d=json.load(open('.flashgrep/index-state.json')); print(len(d.get('files', {})))"
```

## Getting Help

If you continue to experience issues:

1. **Check existing issues**: Search the [GitHub Issues](https://github.com/nnlgsakib/flashgrep/issues)

2. **Collect diagnostic information**:
   ```bash
   # System info
   uname -a
   flashgrep --version
   
   # Repository info
   du -sh .
   find . -type f | wc -l
   
   # Config
   cat .flashgrep/config.json
   ```

3. **Create a minimal reproduction**:
   - Create a small test repository
   - Try to reproduce the issue
   - Share the steps and error messages

## Configuration Reference

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RUST_LOG` | Set logging level (error, warn, info, debug, trace) |
| `FLASHGREP_DIR` | Override the `.flashgrep` directory location |

### Config File Options

See [Configuration Guide](./configuration.md) for all available options.

## Performance Tuning

For large repositories (10,000+ files):

1. **Increase progress interval** (reduce logging overhead):
   ```json
   {"progress_interval": 5000}
   ```

2. **Exclude non-essential files**:
   ```
   # Documentation
   *.md
   *.txt
   
   # Tests (if not needed for search)
   *test*
   *spec*
   ```

3. **Use SSD storage** for the `.flashgrep` directory if possible

4. **Consider disabling initial indexing** if startup time is critical:
   ```json
   {"enable_initial_index": false}
   ```
