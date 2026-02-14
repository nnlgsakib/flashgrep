use crate::FlashgrepResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherEntry {
    pub repo_root: String,
    pub pid: u32,
    pub started_at: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WatcherRegistryData {
    pub entries: HashMap<String, WatcherEntry>,
}

#[derive(Debug)]
pub struct WatcherRegistry {
    path: PathBuf,
    data: WatcherRegistryData,
}

impl WatcherRegistry {
    pub fn load_default() -> FlashgrepResult<Self> {
        let mut dir = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
        dir.push("flashgrep");
        fs::create_dir_all(&dir)?;

        let path = dir.join("watchers.json");
        Self::load_from_path(path)
    }

    pub fn load_from_path(path: PathBuf) -> FlashgrepResult<Self> {
        let data = if path.exists() {
            let raw = fs::read_to_string(&path)?;
            if raw.trim().is_empty() {
                WatcherRegistryData::default()
            } else {
                serde_json::from_str(&raw).unwrap_or_default()
            }
        } else {
            WatcherRegistryData::default()
        };

        Ok(Self { path, data })
    }

    pub fn save(&self) -> FlashgrepResult<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }

    pub fn canonicalize_repo_path(path: &Path) -> FlashgrepResult<PathBuf> {
        let canonical = path.canonicalize()?;
        Ok(normalize_windows_verbatim(canonical))
    }

    pub fn upsert(&mut self, repo_root: &Path, pid: u32) -> FlashgrepResult<()> {
        let key = Self::canonicalize_repo_path(repo_root)?
            .to_string_lossy()
            .to_string();
        self.data.entries.insert(
            key.clone(),
            WatcherEntry {
                repo_root: key,
                pid,
                started_at: Utc::now().timestamp(),
            },
        );
        self.save()
    }

    pub fn remove(&mut self, repo_root: &Path) -> FlashgrepResult<Option<WatcherEntry>> {
        let key = Self::canonicalize_repo_path(repo_root)?
            .to_string_lossy()
            .to_string();
        let removed = self.data.entries.remove(&key);
        self.save()?;
        Ok(removed)
    }

    pub fn get(&self, repo_root: &Path) -> FlashgrepResult<Option<&WatcherEntry>> {
        let key = Self::canonicalize_repo_path(repo_root)?
            .to_string_lossy()
            .to_string();
        Ok(self.data.entries.get(&key))
    }

    pub fn list(&self) -> Vec<&WatcherEntry> {
        self.data.entries.values().collect()
    }

    pub fn cleanup_stale(&mut self) -> FlashgrepResult<usize> {
        let before = self.data.entries.len();
        self.data
            .entries
            .retain(|_, entry| is_process_alive(entry.pid));
        let removed = before.saturating_sub(self.data.entries.len());
        if removed > 0 {
            self.save()?;
        }
        Ok(removed)
    }
}

pub fn is_process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        if pid > i32::MAX as u32 {
            return false;
        }
        let status = std::process::Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "if (Get-Process -Id {} -ErrorAction SilentlyContinue) {{ exit 0 }} else {{ exit 1 }}",
                    pid
                ),
            ])
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(not(windows))]
    {
        let status = std::process::Command::new("sh")
            .args(["-c", &format!("kill -0 {}", pid)])
            .status();
        status.map(|s| s.success()).unwrap_or(false)
    }
}

pub fn kill_process(pid: u32) -> FlashgrepResult<()> {
    #[cfg(windows)]
    {
        let output = std::process::Command::new("cmd")
            .args(["/C", &format!("taskkill /F /PID {}", pid)])
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::FlashgrepError::Task(format!(
                "Failed to stop process {}: {}",
                pid,
                stderr.trim()
            )));
        }
    }

    #[cfg(not(windows))]
    {
        let output = std::process::Command::new("sh")
            .args(["-c", &format!("kill {}", pid)])
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::FlashgrepError::Task(format!(
                "Failed to stop process {}: {}",
                pid,
                stderr.trim()
            )));
        }
    }

    Ok(())
}

fn normalize_windows_verbatim(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
            return PathBuf::from(format!(r"\\{}", stripped));
        }
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_add_remove_roundtrip() -> FlashgrepResult<()> {
        let temp = tempfile::TempDir::new()?;
        let reg_path = temp.path().join("watchers.json");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo)?;

        let mut reg = WatcherRegistry::load_from_path(reg_path)?;
        reg.upsert(&repo, 12345)?;
        assert!(reg.get(&repo)?.is_some());

        reg.remove(&repo)?;
        assert!(reg.get(&repo)?.is_none());
        Ok(())
    }

    #[test]
    fn cleanup_removes_dead_pids() -> FlashgrepResult<()> {
        let temp = tempfile::TempDir::new()?;
        let reg_path = temp.path().join("watchers.json");
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo)?;

        let mut reg = WatcherRegistry::load_from_path(reg_path)?;
        reg.upsert(&repo, u32::MAX)?;
        let removed = reg.cleanup_stale()?;
        assert!(removed >= 1);
        Ok(())
    }
}
