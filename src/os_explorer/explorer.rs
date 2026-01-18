use anyhow::Context;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, path};

pub fn find_file_with_prefix(folder_path: &str, prefix: &str) -> Option<PathBuf> {
    fs::read_dir(folder_path)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(prefix))
        })
        .and_then(|path| path::absolute(path).ok())
}

pub fn map_to_indexed_vec(map: HashMap<u8, String>) -> Vec<Option<String>> {
    let Some(&max_index) = map.keys().max() else {
        return Vec::new();
    };

    let mut vec = vec![None; max_index as usize + 1];

    for (index, value) in map {
        vec[index as usize] = Some(value);
    }

    vec
}

pub fn get_album_name_from_folder_in_path(root_path: &str) -> anyhow::Result<HashMap<u8, String>> {
    Ok(fs::read_dir(root_path)?
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let folder_name = entry.file_name();
            let folder_str = folder_name.to_str()?;

            let (index_str, name) = folder_str.split_once('_')?;
            let index = index_str.parse::<u8>().ok()?;

            Some((index, name.to_owned()))
        })
        .collect())
}

pub fn search_files_in_path(
    root_path: &str,
    prefix: &str,
) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
    let matching_dir = fs::read_dir(root_path)
        .with_context(|| format!("Failed to read directory: {root_path}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(prefix))
        })
        .ok_or_else(|| {
            anyhow::anyhow!("No valid audio folder found that matches the prefix: {prefix}")
        })?;

    let mut file_list: Vec<PathBuf> = fs::read_dir(&matching_dir)
        .with_context(|| format!("Failed to read directory: {}", matching_dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter_map(|p| path::absolute(p).ok())
        .collect();

    file_list.sort();

    Ok((matching_dir, file_list))
}
