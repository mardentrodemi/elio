use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::filesystem::SortMode;

/// Sort modes chosen with `s`, keyed by the folder's canonical path.
///
/// A folder is stored only while its sort differs from the current
/// `[ui] default_sort`. Returning to that default removes the entry.
/// Changing the default does not rewrite this file.
#[derive(Clone, Debug, Default)]
pub(crate) struct FolderSortMemory {
    entries: BTreeMap<PathBuf, SortMode>,
    path: Option<PathBuf>,
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
struct FolderSortFile {
    #[serde(default)]
    folders: BTreeMap<String, SortMode>,
}

impl FolderSortMemory {
    pub(crate) fn load() -> Self {
        let path = store_path();
        let entries = path.as_deref().map(read_entries).unwrap_or_default();
        Self { entries, path }
    }

    pub(crate) fn sort_for(&self, directory: &Path) -> SortMode {
        self.entries
            .get(&directory_key(directory))
            .copied()
            .unwrap_or_else(|| crate::config::ui().default_sort)
    }

    /// Remember `sort` for `directory`, or forget it when it matches the default.
    pub(crate) fn record(&mut self, directory: &Path, sort: SortMode) {
        let key = directory_key(directory);
        let changed = if sort == crate::config::ui().default_sort {
            self.entries.remove(&key).is_some()
        } else if self.entries.get(&key) == Some(&sort) {
            false
        } else {
            self.entries.insert(key, sort);
            true
        };
        if changed {
            self.save();
        }
    }

    fn save(&self) {
        let Some(path) = &self.path else {
            return;
        };
        if let Err(error) = write_entries(path, &self.entries) {
            eprintln!(
                "elio: failed to save folder sort to {}: {error}",
                path.display()
            );
        }
    }
}

fn store_path() -> Option<PathBuf> {
    // Unit tests construct a full app and must not read or write the user's sorts.
    if cfg!(test) {
        return None;
    }
    crate::config::config_dir().map(|dir| dir.join("folder_sort.toml"))
}

fn directory_key(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn read_entries(path: &Path) -> BTreeMap<PathBuf, SortMode> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return BTreeMap::new(),
        Err(error) => {
            eprintln!(
                "elio: failed to read folder sort from {}: {error}",
                path.display()
            );
            return BTreeMap::new();
        }
    };
    let parsed: FolderSortFile = match toml::from_str(&contents) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!(
                "elio: failed to load folder sort from {}: {error}",
                path.display()
            );
            return BTreeMap::new();
        }
    };
    parsed
        .folders
        .into_iter()
        .map(|(folder, sort)| (PathBuf::from(folder), sort))
        .collect()
}

fn write_entries(path: &Path, entries: &BTreeMap<PathBuf, SortMode>) -> io::Result<()> {
    if entries.is_empty() {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = FolderSortFile {
            folders: entries
                .iter()
                .map(|(folder, sort)| (folder.to_string_lossy().into_owned(), *sort))
                .collect(),
        };
        let contents = toml::to_string_pretty(&file)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;
        let temporary = path.with_extension("toml.tmp");
        fs::write(&temporary, contents)?;
        fs::rename(&temporary, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(label: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("elio-folder-sort-{label}-{unique}.toml"))
    }

    fn sort_token(sort: SortMode) -> &'static str {
        match sort {
            SortMode::Name => "name",
            SortMode::Modified => "modified",
            SortMode::Size => "size",
        }
    }

    #[test]
    fn record_keeps_only_sorts_that_differ_from_the_default() {
        let path = temp_store("record");
        let downloads = std::env::temp_dir().join(format!(
            "elio-folder-sort-downloads-{}",
            path.file_stem().unwrap().to_string_lossy()
        ));
        fs::create_dir_all(&downloads).unwrap();
        let default = crate::config::ui().default_sort;
        let chosen = default.cycle();
        let mut memory = FolderSortMemory {
            entries: BTreeMap::new(),
            path: Some(path.clone()),
        };

        memory.record(&downloads, default);
        assert!(!path.exists());
        assert_eq!(memory.sort_for(&downloads), default);

        memory.record(&downloads, chosen);
        let saved = fs::read_to_string(&path).unwrap();
        assert!(saved.contains(sort_token(chosen)));
        assert!(!saved.contains(&format!("\"{}\"", sort_token(default))));

        let reloaded = FolderSortMemory {
            entries: read_entries(&path),
            path: Some(path.clone()),
        };
        assert_eq!(reloaded.sort_for(&downloads), chosen);

        memory.record(&downloads, default);
        assert!(!path.exists());
        assert_eq!(memory.sort_for(&downloads), default);
        fs::remove_dir_all(&downloads).unwrap();
    }

    #[test]
    fn loading_keeps_an_entry_that_matches_the_current_default() {
        let path = temp_store("stale-default");
        let downloads = std::env::temp_dir().join(format!(
            "elio-folder-sort-stale-{}",
            path.file_stem().unwrap().to_string_lossy()
        ));
        fs::create_dir_all(&downloads).unwrap();
        let default = crate::config::ui().default_sort;
        let key = fs::canonicalize(&downloads).unwrap();
        fs::write(
            &path,
            format!(
                "[folders]\n\"{}\" = \"{}\"\n",
                key.display(),
                sort_token(default)
            ),
        )
        .unwrap();

        let memory = FolderSortMemory {
            entries: read_entries(&path),
            path: Some(path.clone()),
        };
        assert_eq!(memory.sort_for(&downloads), default);
        assert!(path.exists());
        let _ = fs::remove_file(&path);
        fs::remove_dir_all(&downloads).unwrap();
    }
}
