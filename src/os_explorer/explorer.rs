use anyhow::Context;
use std::path::{Path, PathBuf};
use std::{fs, path};

/// A music album: a subfolder of the music folder, assigned to one pad.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    pub path: PathBuf,
    pub label: String,
}

/// Splits `08_Tavern` into `(Some(8), "Tavern")`; names without a numeric
/// prefix are returned whole.
fn split_prefix(name: &str) -> (Option<u32>, &str) {
    name.split_once('_')
        .filter(|(index, rest)| {
            !index.is_empty() && !rest.is_empty() && index.bytes().all(|b| b.is_ascii_digit())
        })
        .and_then(|(index, rest)| index.parse().ok().map(|i| (Some(i), rest)))
        .unwrap_or((None, name))
}

/// Album folders in pad order: pad 0 is the first album, pad 1 the second…
///
/// Folders with a numeric prefix come first, ordered by that number (so
/// `2_Town` precedes `10_Sad` without zero padding), ties broken by name.
/// Folders without a prefix follow, by name.
pub fn list_albums(root_path: &str) -> anyhow::Result<Vec<Album>> {
    let mut albums: Vec<(Option<u32>, String, Album)> = fs::read_dir(root_path)
        .with_context(|| format!("Failed to read directory: {root_path}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.to_owned();
            let (index, label) = split_prefix(&name);
            let label = label.to_owned();
            Some((index, label.to_lowercase(), Album { path, label }))
        })
        .collect();

    albums.sort_by(|(ia, la, a), (ib, lb, b)| {
        (ia.is_none(), ia, la, &a.path).cmp(&(ib.is_none(), ib, lb, &b.path))
    });
    Ok(albums.into_iter().map(|(_, _, album)| album).collect())
}

/// The album assigned to pad `index` and its audio files.
pub fn nth_album_files(root_path: &str, index: usize) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
    let album = list_albums(root_path)?
        .into_iter()
        .nth(index)
        .ok_or_else(|| anyhow::anyhow!("No album for pad {index} in {root_path}"))?;
    let files = get_all_files_in_folder(&album.path)?;
    Ok((album.path, files))
}

pub fn get_all_files_in_folder(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut file_list: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter_map(|p| path::absolute(p).ok())
        .collect();

    file_list.sort();
    Ok(file_list)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempLibrary(PathBuf);

    impl TempLibrary {
        fn new(test: &str, folders: &[&str]) -> Self {
            let root = std::env::temp_dir().join(format!("teatro-{test}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            for f in folders {
                fs::create_dir_all(root.join(f)).expect("create test folder");
            }
            fs::write(root.join("stray-file.opus"), b"").expect("create test file");
            Self(root)
        }

        fn path(&self) -> &str {
            self.0.to_str().expect("utf-8 temp path")
        }
    }

    impl Drop for TempLibrary {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn labels(lib: &TempLibrary) -> Vec<String> {
        list_albums(lib.path())
            .expect("list albums")
            .into_iter()
            .map(|a| a.label)
            .collect()
    }

    #[test]
    fn prefix_parsing() {
        assert_eq!(split_prefix("08_Tavern"), (Some(8), "Tavern"));
        assert_eq!(split_prefix("13_veil_end"), (Some(13), "veil_end"));
        assert_eq!(split_prefix("Tavern"), (None, "Tavern"));
        assert_eq!(split_prefix("x_Tavern"), (None, "x_Tavern"));
        assert_eq!(split_prefix("12_"), (None, "12_"));
    }

    #[test]
    fn numeric_order_without_zero_padding() {
        let lib = TempLibrary::new("numeric", &["10_Sad", "2_Town", "1_Chase", "0_Stealth"]);
        assert_eq!(labels(&lib), ["Stealth", "Chase", "Town", "Sad"]);
    }

    #[test]
    fn duplicates_and_gaps_keep_every_album() {
        let lib = TempLibrary::new(
            "dupes",
            &["11_Death", "13_veil", "16_farewell", "16_Siege", "Extras"],
        );
        assert_eq!(
            labels(&lib),
            ["Death", "veil", "farewell", "Siege", "Extras"]
        );

        let albums = list_albums(lib.path()).expect("list albums");
        for (pad, album) in albums.iter().enumerate() {
            let (dir, _) = nth_album_files(lib.path(), pad).expect("album for pad");
            assert_eq!(
                dir, album.path,
                "pad {pad} plays a different album than it shows"
            );
        }
        assert!(nth_album_files(lib.path(), albums.len()).is_err());
    }
}
