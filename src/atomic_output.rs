use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempPath};

struct StagedOutput {
    target_path: PathBuf,
    temporary_path: TempPath,
}

pub(crate) struct AtomicOutputSet {
    output_directory: PathBuf,
    staged_outputs: Vec<StagedOutput>,
    removal_targets: Vec<PathBuf>,
    target_paths: HashSet<PathBuf>,
}

impl AtomicOutputSet {
    pub(crate) fn new(output_directory: &Path) -> Result<Self> {
        fs::create_dir_all(output_directory).with_context(|| {
            format!(
                "Failed to create output directory {}",
                output_directory.display()
            )
        })?;

        Ok(Self {
            output_directory: output_directory.to_path_buf(),
            staged_outputs: Vec::new(),
            removal_targets: Vec::new(),
            target_paths: HashSet::new(),
        })
    }

    pub(crate) fn stage<F>(&mut self, target_path: &Path, writer: F) -> Result<PathBuf>
    where
        F: FnOnce(&Path) -> Result<()>,
    {
        let parent_directory = parent_directory(target_path);
        if parent_directory != self.output_directory {
            bail!(
                "Atomic output target {} is not in transaction directory {}",
                target_path.display(),
                self.output_directory.display()
            );
        }
        if !self.target_paths.insert(target_path.to_path_buf()) {
            bail!("Duplicate atomic output target: {}", target_path.display());
        }

        let temporary_file = Builder::new()
            .prefix(".methrix-")
            .suffix(".part")
            .tempfile_in(parent_directory)
            .with_context(|| {
                format!(
                    "Failed to create temporary output beside {}",
                    target_path.display()
                )
            })?;
        let temporary_path = temporary_file.path().to_path_buf();

        if let Err(error) = writer(&temporary_path) {
            self.target_paths.remove(target_path);
            return Err(error).with_context(|| {
                format!(
                    "Failed while staging atomic output {}",
                    target_path.display()
                )
            });
        }

        sync_file(&temporary_path)?;
        let temporary_path = temporary_file.into_temp_path();
        self.staged_outputs.push(StagedOutput {
            target_path: target_path.to_path_buf(),
            temporary_path,
        });

        Ok(temporary_path_for_target(
            self.staged_outputs
                .last()
                .context("Staged output unexpectedly missing")?,
        ))
    }

    pub(crate) fn remove(&mut self, target_path: &Path) -> Result<()> {
        let parent_directory = parent_directory(target_path);
        if parent_directory != self.output_directory {
            bail!(
                "Atomic removal target {} is not in transaction directory {}",
                target_path.display(),
                self.output_directory.display()
            );
        }
        if !self.target_paths.insert(target_path.to_path_buf()) {
            bail!("Duplicate atomic output target: {}", target_path.display());
        }
        self.removal_targets.push(target_path.to_path_buf());
        Ok(())
    }

    pub(crate) fn publish(self) -> Result<()> {
        if self.staged_outputs.is_empty() && self.removal_targets.is_empty() {
            return Ok(());
        }

        let backup_directory = Builder::new()
            .prefix(".methrix-backup-")
            .tempdir_in(&self.output_directory)
            .context("Failed to create atomic output backup directory")?;
        let mut backups = Vec::new();

        let transaction_targets: Vec<&PathBuf> = self
            .staged_outputs
            .iter()
            .map(|staged_output| &staged_output.target_path)
            .chain(self.removal_targets.iter())
            .collect();
        for (output_index, target_path) in transaction_targets.iter().enumerate() {
            if target_path.exists() {
                let backup_path = backup_directory.path().join(output_index.to_string());
                if let Err(error) = fs::rename(target_path, &backup_path) {
                    restore_backups(&backups);
                    return Err(error).with_context(|| {
                        format!(
                            "Failed to preserve existing output {}",
                            target_path.display()
                        )
                    });
                }
                backups.push(((*target_path).clone(), backup_path));
            }
        }

        let mut published_targets = Vec::new();
        for staged_output in &self.staged_outputs {
            let temporary_path: &Path = staged_output.temporary_path.as_ref();
            if let Err(error) = fs::rename(temporary_path, &staged_output.target_path) {
                rollback_publication(&published_targets, &backups);
                return Err(error).with_context(|| {
                    format!(
                        "Failed to publish atomic output {}",
                        staged_output.target_path.display()
                    )
                });
            }
            published_targets.push(staged_output.target_path.clone());
        }

        sync_directory(&self.output_directory)?;
        Ok(())
    }
}

pub(crate) fn write_atomically<F>(target_path: &Path, writer: F) -> Result<()>
where
    F: FnOnce(&Path) -> Result<()>,
{
    let output_directory = parent_directory(target_path);
    let mut output_set = AtomicOutputSet::new(output_directory)?;
    output_set.stage(target_path, writer)?;
    output_set.publish()
}

fn parent_directory(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn temporary_path_for_target(staged_output: &StagedOutput) -> PathBuf {
    let temporary_path: &Path = staged_output.temporary_path.as_ref();
    temporary_path.to_path_buf()
}

fn sync_file(path: &Path) -> Result<()> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .with_context(|| format!("Failed to reopen staged output {}", path.display()))?
        .sync_all()
        .with_context(|| format!("Failed to sync staged output {}", path.display()))
}

fn restore_backups(backups: &[(PathBuf, PathBuf)]) {
    for (target_path, backup_path) in backups.iter().rev() {
        let _ = fs::rename(backup_path, target_path);
    }
}

fn rollback_publication(published_targets: &[PathBuf], backups: &[(PathBuf, PathBuf)]) {
    for target_path in published_targets.iter().rev() {
        let _ = fs::remove_file(target_path);
    }
    restore_backups(backups);
}

#[cfg(unix)]
fn sync_directory(directory: &Path) -> Result<()> {
    File::open(directory)
        .with_context(|| format!("Failed to open output directory {}", directory.display()))?
        .sync_all()
        .with_context(|| format!("Failed to sync output directory {}", directory.display()))
}

#[cfg(not(unix))]
fn sync_directory(_directory: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{write_atomically, AtomicOutputSet};
    use anyhow::bail;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn failed_staging_preserves_existing_output_set() {
        let temporary_directory = tempdir().unwrap();
        let first_target = temporary_directory.path().join("first.txt");
        let second_target = temporary_directory.path().join("second.txt");
        fs::write(&first_target, "old-first").unwrap();
        fs::write(&second_target, "old-second").unwrap();

        let mut output_set = AtomicOutputSet::new(temporary_directory.path()).unwrap();
        output_set
            .stage(&first_target, |temporary_path| {
                fs::write(temporary_path, "new-first")?;
                Ok(())
            })
            .unwrap();
        let staging_result = output_set.stage(&second_target, |_temporary_path| {
            bail!("simulated writer failure")
        });

        assert!(staging_result.is_err());
        assert_eq!(fs::read_to_string(first_target).unwrap(), "old-first");
        assert_eq!(fs::read_to_string(second_target).unwrap(), "old-second");
    }

    #[test]
    fn failed_mid_publication_restores_all_existing_outputs() {
        let temporary_directory = tempdir().unwrap();
        let first_target = temporary_directory.path().join("first.txt");
        let second_target = temporary_directory.path().join("second.txt");
        fs::write(&first_target, "old-first").unwrap();
        fs::write(&second_target, "old-second").unwrap();

        let mut output_set = AtomicOutputSet::new(temporary_directory.path()).unwrap();
        output_set
            .stage(&first_target, |temporary_path| {
                fs::write(temporary_path, "new-first")?;
                Ok(())
            })
            .unwrap();
        let second_temporary_path = output_set
            .stage(&second_target, |temporary_path| {
                fs::write(temporary_path, "new-second")?;
                Ok(())
            })
            .unwrap();
        fs::remove_file(second_temporary_path).unwrap();

        let publication_result = output_set.publish();

        assert!(publication_result.is_err());
        assert_eq!(fs::read_to_string(first_target).unwrap(), "old-first");
        assert_eq!(fs::read_to_string(second_target).unwrap(), "old-second");
    }

    #[test]
    fn atomically_removes_stale_output_with_replacement_set() {
        let temporary_directory = tempdir().unwrap();
        let replacement_target = temporary_directory.path().join("replacement.txt");
        let stale_target = temporary_directory.path().join("stale.txt");
        fs::write(&replacement_target, "old").unwrap();
        fs::write(&stale_target, "stale").unwrap();

        let mut output_set = AtomicOutputSet::new(temporary_directory.path()).unwrap();
        output_set
            .stage(&replacement_target, |temporary_path| {
                fs::write(temporary_path, "new")?;
                Ok(())
            })
            .unwrap();
        output_set.remove(&stale_target).unwrap();
        output_set.publish().unwrap();

        assert_eq!(fs::read_to_string(replacement_target).unwrap(), "new");
        assert!(!stale_target.exists());
    }

    #[test]
    fn atomically_replaces_existing_file() {
        let temporary_directory = tempdir().unwrap();
        let target_path = temporary_directory.path().join("output.txt");
        fs::write(&target_path, "old").unwrap();

        write_atomically(&target_path, |temporary_path| {
            fs::write(temporary_path, "new")?;
            Ok(())
        })
        .unwrap();

        assert_eq!(fs::read_to_string(target_path).unwrap(), "new");
    }
}
