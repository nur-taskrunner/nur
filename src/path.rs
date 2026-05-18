use std::fs;
use std::io::Result;
use std::path::{Path, PathBuf};

/// Get the directory where the Nushell executable is located.
fn current_exe_directory() -> PathBuf {
    let mut path = std::env::current_exe().expect("current_exe() should succeed");
    path.pop();
    path
}

/// Get the current working directory from the environment.
pub(crate) fn current_dir_from_environment() -> PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        return cwd;
    }
    if let Ok(cwd) = std::env::var("PWD") {
        return cwd.into();
    }
    if let Some(home) = nu_path::home_dir() {
        return home.into_std_path_buf();
    }
    current_exe_directory()
}

pub(crate) fn find_project_path<P: AsRef<Path>>(
    cwd: P,
    nurfile_names: &Vec<String>,
) -> Option<PathBuf> {
    let mut path = cwd.as_ref();

    loop {
        if find_nurfile(path, nurfile_names).is_some() {
            return Some(path.to_path_buf());
        }

        if let Some(parent) = path.parent() {
            path = parent;
        } else {
            return None;
        }
    }
}

pub(crate) fn find_nurfile<P: AsRef<Path>>(
    project_path_: P,
    nurfile_names: &Vec<String>,
) -> Option<PathBuf> {
    let project_path = project_path_.as_ref();

    for nur_file_name in nurfile_names {
        let nur_file_path = project_path.join(nur_file_name);
        if nur_file_path.exists() {
            return Some(nur_file_path);
        }
    }

    None
}

// Copy of nushell: src/config_files.rs -> read_and_sort_directory
pub(crate) fn read_and_sort_directory(path: &Path) -> Result<Vec<String>> {
    let mut entries = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_name_str = file_name.into_string().unwrap_or_default();
        entries.push(file_name_str);
    }

    entries.sort();

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, create_dir};
    use tempfile::tempdir;

    use crate::names::{NUR_FILE, NUR_FILE_DOT_NU};

    #[test]
    fn test_find_project_path() {
        // Create a temporary directory and a "nurfile" inside it
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let nurfile_path = temp_dir.path().join(NUR_FILE);
        File::create(&nurfile_path).unwrap();

        // Test the function with the temporary directory as the current working directory
        let expected_path = temp_dir_path.clone();
        let actual_path = find_project_path(
            &temp_dir_path,
            &vec![String::from(NUR_FILE), String::from(NUR_FILE_DOT_NU)],
        )
        .unwrap();
        assert_eq!(expected_path, actual_path);

        // Clean up
        std::fs::remove_file(nurfile_path).unwrap();
    }

    #[test]
    fn test_find_project_path_subdirectory() {
        // Create a temporary directory and a subdirectory inside it
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let sub_dir = temp_dir_path.join("sub");
        create_dir(&sub_dir).unwrap();

        // Create a "nurfile" inside the temporary directory
        let nurfile_path = temp_dir_path.join(NUR_FILE);
        File::create(&nurfile_path).unwrap();

        // Test the function with the subdirectory as the current working directory
        let expected_path = temp_dir_path.clone();
        let actual_path = find_project_path(
            &temp_dir_path,
            &vec![String::from(NUR_FILE), String::from(NUR_FILE_DOT_NU)],
        )
        .unwrap();
        assert_eq!(expected_path, actual_path);

        // Clean up
        std::fs::remove_file(nurfile_path).unwrap();
        std::fs::remove_dir(sub_dir).unwrap();
    }

    #[test]
    fn test_find_project_path_with_nurfile_dot_nu_subdirectory() {
        // Create a temporary directory and a subdirectory inside it
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let sub_dir = temp_dir_path.join("sub");
        create_dir(&sub_dir).unwrap();

        // Create a "nurfile" inside the temporary directory
        let nurfile_path = temp_dir_path.join(NUR_FILE_DOT_NU);
        File::create(&nurfile_path).unwrap();

        // Test the function with the subdirectory as the current working directory
        let expected_path = temp_dir_path.clone();
        let actual_path = find_project_path(
            &temp_dir_path,
            &vec![String::from(NUR_FILE), String::from(NUR_FILE_DOT_NU)],
        )
        .unwrap();
        assert_eq!(expected_path, actual_path);

        // Clean up
        std::fs::remove_file(nurfile_path).unwrap();
        std::fs::remove_dir(sub_dir).unwrap();
    }

    #[test]
    fn test_find_project_path_error() {
        // Create a temporary directory without a "nurfile"
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();

        // Test the function with the temporary directory as the current working directory
        match find_project_path(
            &temp_dir_path,
            &vec![String::from(NUR_FILE), String::from(NUR_FILE_DOT_NU)],
        ) {
            Some(_) => panic!("Expected an error, but got Ok"),
            None => (),
        }
    }
}
