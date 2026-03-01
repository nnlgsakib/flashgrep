use super::OutputMode;
use crate::path_utils::resolve_path;
use crate::{FlashgrepError, FlashgrepResult};
use clap::Subcommand;
use serde::Serialize;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Subcommand)]
pub enum FsCommands {
    /// Create a file or directory
    Create {
        /// Target path
        path: PathBuf,
        /// Create directory instead of file
        #[arg(long)]
        dir: bool,
        /// Create parent directories if missing
        #[arg(long)]
        parents: bool,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// List entries in a directory
    List {
        /// Directory path to list
        path: PathBuf,
        /// Include hidden entries
        #[arg(long = "include-hidden")]
        include_hidden: bool,
        /// Sort by path, name, modified, or size
        #[arg(long = "sort-by", default_value = "path")]
        sort_by: String,
        /// Sort order asc or desc
        #[arg(long = "sort-order", default_value = "asc")]
        sort_order: String,
        /// Limit returned entries
        #[arg(long)]
        limit: Option<usize>,
        /// Offset for deterministic pagination
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Stat a file or directory
    Stat {
        /// Target path
        path: PathBuf,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Copy files or directories
    Copy {
        /// Source path
        src: PathBuf,
        /// Destination path
        dst: PathBuf,
        /// Copy directories recursively
        #[arg(long)]
        recursive: bool,
        /// Overwrite destination if it exists
        #[arg(long)]
        overwrite: bool,
        /// Show operations without applying
        #[arg(long)]
        dry_run: bool,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Move files or directories
    Move {
        /// Source path
        src: PathBuf,
        /// Destination path
        dst: PathBuf,
        /// Overwrite destination if it exists
        #[arg(long)]
        overwrite: bool,
        /// Show operations without applying
        #[arg(long)]
        dry_run: bool,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
    /// Remove files or directories
    Remove {
        /// Target path
        path: PathBuf,
        /// Remove directory recursively
        #[arg(short = 'r', long)]
        recursive: bool,
        /// Force non-interactive removal
        #[arg(short, long)]
        force: bool,
        /// Show operations without applying
        #[arg(long)]
        dry_run: bool,
        /// Output format
        #[arg(long, value_enum, default_value_t = OutputMode::Text)]
        output: OutputMode,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct FsEntry {
    pub path: String,
    pub file_type: String,
    pub size: u64,
    pub modified_unix: Option<u64>,
    pub readonly: bool,
}

pub fn handle_fs_command(repo_root: &Path, cmd: FsCommands) -> FlashgrepResult<()> {
    match cmd {
        FsCommands::Create {
            path,
            dir,
            parents,
            output,
        } => {
            let target = resolve_path(repo_root, &path);
            if dir {
                if parents {
                    fs::create_dir_all(&target)?;
                } else {
                    fs::create_dir(&target)?;
                }
            } else {
                if parents {
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                }
                if !target.exists() {
                    fs::File::create(&target)?;
                }
            }
            render_single(output, &FsEntry::from_path(&target)?, "create", false, None)
        }
        FsCommands::List {
            path,
            include_hidden,
            sort_by,
            sort_order,
            limit,
            offset,
            output,
        } => {
            let target = resolve_path(repo_root, &path);
            if !target.is_dir() {
                return Err(FlashgrepError::Config(format!(
                    "List target is not a directory: {}",
                    target.display()
                )));
            }

            let mut entries = Vec::new();
            for entry in fs::read_dir(&target)? {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                if !include_hidden && name.starts_with('.') {
                    continue;
                }
                entries.push(FsEntry::from_path(&entry.path())?);
            }

            sort_entries(&mut entries, &sort_by, &sort_order)?;
            let total = entries.len();
            let mut window: Vec<FsEntry> = entries.into_iter().skip(offset).collect();
            if let Some(lim) = limit {
                window.truncate(lim);
            }
            let next_offset = if offset.saturating_add(window.len()) < total {
                Some(offset.saturating_add(window.len()))
            } else {
                None
            };
            render_many(output, &window, "list", false, next_offset, Some(total))
        }
        FsCommands::Stat { path, output } => {
            let target = resolve_path(repo_root, &path);
            render_single(output, &FsEntry::from_path(&target)?, "stat", false, None)
        }
        FsCommands::Copy {
            src,
            dst,
            recursive,
            overwrite,
            dry_run,
            output,
        } => {
            let src = resolve_path(repo_root, &src);
            let dst = resolve_path(repo_root, &dst);
            if !src.exists() {
                return Err(FlashgrepError::Config(format!(
                    "Source does not exist: {}",
                    src.display()
                )));
            }
            if dst.exists() && !overwrite {
                return Err(FlashgrepError::Config(format!(
                    "Destination already exists: {} (use --overwrite)",
                    dst.display()
                )));
            }

            if !dry_run {
                if src.is_dir() {
                    if !recursive {
                        return Err(FlashgrepError::Config(
                            "Source is a directory. Use --recursive for directory copy".to_string(),
                        ));
                    }
                    copy_dir_recursive(&src, &dst, overwrite)?;
                } else {
                    if let Some(parent) = dst.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let _ = fs::copy(&src, &dst)?;
                }
            }

            render_single(
                output,
                &FsEntry::from_path(&src)?,
                "copy",
                dry_run,
                Some(dst.to_string_lossy().to_string()),
            )
        }
        FsCommands::Move {
            src,
            dst,
            overwrite,
            dry_run,
            output,
        } => {
            let src = resolve_path(repo_root, &src);
            let dst = resolve_path(repo_root, &dst);
            if !src.exists() {
                return Err(FlashgrepError::Config(format!(
                    "Source does not exist: {}",
                    src.display()
                )));
            }
            if dst.exists() {
                if !overwrite {
                    return Err(FlashgrepError::Config(format!(
                        "Destination already exists: {} (use --overwrite)",
                        dst.display()
                    )));
                }
                if !dry_run {
                    if dst.is_dir() {
                        fs::remove_dir_all(&dst)?;
                    } else {
                        fs::remove_file(&dst)?;
                    }
                }
            }

            if !dry_run {
                if let Some(parent) = dst.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::rename(&src, &dst)?;
            }

            let entry = if dry_run {
                FsEntry::from_path(&src)?
            } else {
                FsEntry::from_path(&dst)?
            };
            render_single(
                output,
                &entry,
                "move",
                dry_run,
                Some(dst.to_string_lossy().to_string()),
            )
        }
        FsCommands::Remove {
            path,
            recursive,
            force,
            dry_run,
            output,
        } => {
            let target = resolve_path(repo_root, &path);
            if !target.exists() {
                if force {
                    return render_single(
                        output,
                        &FsEntry {
                            path: target.to_string_lossy().to_string(),
                            file_type: "missing".to_string(),
                            size: 0,
                            modified_unix: None,
                            readonly: false,
                        },
                        "remove",
                        dry_run,
                        None,
                    );
                }
                return Err(FlashgrepError::Config(format!(
                    "Target does not exist: {}",
                    target.display()
                )));
            }

            let entry = FsEntry::from_path(&target)?;
            if !dry_run {
                if target.is_dir() {
                    if recursive {
                        fs::remove_dir_all(&target)?;
                    } else {
                        fs::remove_dir(&target).map_err(|_| {
                            FlashgrepError::Config(format!(
                                "Directory removal requires --recursive when non-empty: {}",
                                target.display()
                            ))
                        })?;
                    }
                } else {
                    fs::remove_file(&target)?;
                }
            }
            render_single(output, &entry, "remove", dry_run, None)
        }
    }
}

impl FsEntry {
    fn from_path(path: &Path) -> FlashgrepResult<Self> {
        let metadata = fs::metadata(path)?;
        let file_type = if metadata.is_dir() {
            "directory"
        } else {
            "file"
        };
        let modified_unix = metadata
            .modified()
            .ok()
            .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        Ok(Self {
            path: path.to_string_lossy().to_string(),
            file_type: file_type.to_string(),
            size: metadata.len(),
            modified_unix,
            readonly: metadata.permissions().readonly(),
        })
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path, overwrite: bool) -> FlashgrepResult<()> {
    fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.map_err(|e| FlashgrepError::Config(e.to_string()))?;
        let rel = entry.path().strip_prefix(src).map_err(|e| {
            FlashgrepError::Config(format!("Failed to compute relative path: {}", e))
        })?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if target.exists() && !overwrite {
                return Err(FlashgrepError::Config(format!(
                    "Destination already exists: {} (use --overwrite)",
                    target.display()
                )));
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let _ = fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

fn sort_entries(entries: &mut [FsEntry], sort_by: &str, sort_order: &str) -> FlashgrepResult<()> {
    if !matches!(sort_by, "path" | "name" | "modified" | "size") {
        return Err(FlashgrepError::Config(format!(
            "Invalid sort-by '{}'. Expected path, name, modified, or size",
            sort_by
        )));
    }
    if sort_order != "asc" && sort_order != "desc" {
        return Err(FlashgrepError::Config(format!(
            "Invalid sort-order '{}'. Expected asc or desc",
            sort_order
        )));
    }

    entries.sort_by(|a, b| {
        let ord = match sort_by {
            "path" => a.path.cmp(&b.path),
            "name" => {
                let a_name = Path::new(&a.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let b_name = Path::new(&b.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                a_name.cmp(&b_name).then_with(|| a.path.cmp(&b.path))
            }
            "modified" => a
                .modified_unix
                .unwrap_or(0)
                .cmp(&b.modified_unix.unwrap_or(0))
                .then_with(|| a.path.cmp(&b.path)),
            "size" => a.size.cmp(&b.size).then_with(|| a.path.cmp(&b.path)),
            _ => Ordering::Equal,
        };
        match sort_order {
            "asc" => ord,
            "desc" => match ord {
                Ordering::Less => Ordering::Greater,
                Ordering::Greater => Ordering::Less,
                Ordering::Equal => Ordering::Equal,
            },
            _ => ord,
        }
    });

    Ok(())
}

fn render_single(
    output: OutputMode,
    entry: &FsEntry,
    op: &str,
    dry_run: bool,
    destination: Option<String>,
) -> FlashgrepResult<()> {
    match output {
        OutputMode::Json => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "operation": op,
                    "dry_run": dry_run,
                    "entry": entry,
                    "destination": destination,
                }))?
            );
        }
        OutputMode::Text => {
            println!("{}: {}", op, entry.path);
            println!(
                "  type={} size={} readonly={}",
                entry.file_type, entry.size, entry.readonly
            );
            if let Some(modified) = entry.modified_unix {
                println!("  modified_unix={}", modified);
            }
            if let Some(dst) = destination {
                println!("  destination={}", dst);
            }
            if dry_run {
                println!("  dry_run=true");
            }
        }
    }
    Ok(())
}

fn render_many(
    output: OutputMode,
    entries: &[FsEntry],
    op: &str,
    dry_run: bool,
    next_offset: Option<usize>,
    total: Option<usize>,
) -> FlashgrepResult<()> {
    match output {
        OutputMode::Json => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "operation": op,
                    "dry_run": dry_run,
                    "entries": entries,
                    "total": total,
                    "returned": entries.len(),
                    "next_offset": next_offset,
                    "completed": next_offset.is_none(),
                }))?
            );
        }
        OutputMode::Text => {
            println!("{}: {} entrie(s)", op, entries.len());
            for entry in entries {
                println!(
                    "- {} type={} size={}",
                    entry.path, entry.file_type, entry.size
                );
            }
            if let Some(next) = next_offset {
                println!("next_offset={}", next);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn fs_create_and_stat_file() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path().to_path_buf();
        handle_fs_command(
            &root,
            FsCommands::Create {
                path: PathBuf::from("a.txt"),
                dir: false,
                parents: false,
                output: OutputMode::Json,
            },
        )
        .expect("create");
        assert!(root.join("a.txt").exists());
    }

    #[test]
    fn fs_remove_dry_run_keeps_target() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path().to_path_buf();
        std::fs::write(root.join("a.txt"), "x").expect("seed");
        handle_fs_command(
            &root,
            FsCommands::Remove {
                path: PathBuf::from("a.txt"),
                recursive: false,
                force: false,
                dry_run: true,
                output: OutputMode::Json,
            },
        )
        .expect("remove dry run");
        assert!(root.join("a.txt").exists());
    }

    #[test]
    fn fs_copy_requires_overwrite_when_destination_exists() {
        let temp = TempDir::new().expect("temp dir");
        let root = temp.path().to_path_buf();
        std::fs::write(root.join("src.txt"), "src").expect("src");
        std::fs::write(root.join("dst.txt"), "dst").expect("dst");
        let err = handle_fs_command(
            &root,
            FsCommands::Copy {
                src: PathBuf::from("src.txt"),
                dst: PathBuf::from("dst.txt"),
                recursive: false,
                overwrite: false,
                dry_run: false,
                output: OutputMode::Json,
            },
        )
        .expect_err("overwrite error");
        assert!(err.to_string().contains("Destination already exists"));
    }
}
