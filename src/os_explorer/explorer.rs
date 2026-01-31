use anyhow::Context;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, path};

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
        .filter_map(Result::ok)
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

pub fn files_in_nth_subdir(
    root_path: &str,
    folder_index: usize,
) -> anyhow::Result<(PathBuf, Vec<PathBuf>)> {
    let mut list_of_dirs: Vec<_> = fs::read_dir(root_path)
        .with_context(|| format!("Failed to read directory: {root_path}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|path| path.is_dir() && path.file_name().and_then(|n| n.to_str()).is_some())
        .collect();

    list_of_dirs.sort();

    let matching_dir = list_of_dirs.get(folder_index).ok_or_else(|| {
        anyhow::anyhow!("Invalid folder index: {folder_index} in path {root_path}")
    })?;
    let file_list = get_all_files_in_folder(matching_dir)?;
    Ok((matching_dir.clone(), file_list))
}

pub fn get_all_files_in_folder(dir: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
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
