use crate::names::NUR_FILE;
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

pub(crate) fn find_project_path<P: AsRef<Path>>(cwd: P) -> Option<PathBuf> {
    let mut path = cwd.as_ref();

    loop {
        let taskfile_path = path.join(NUR_FILE);
        if taskfile_path.exists() {
            return Some(path.to_path_buf());
        }

        if let Some(parent) = path.parent() {
            path = parent;
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, create_dir};
    use tempfile::tempdir;

    #[test]
    fn test_find_project_path() {
        // Create a temporary directory and a "nurfile" inside it
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let nurfile_path = temp_dir.path().join(NUR_FILE);
        File::create(&nurfile_path).unwrap();

        // Test the function with the temporary directory as the current working directory
        let expected_path = temp_dir_path.clone();
        let actual_path = find_project_path(&temp_dir_path).unwrap();
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
        let actual_path = find_project_path(&sub_dir).unwrap();
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
        match find_project_path(&temp_dir_path) {
            Some(_) => panic!("Expected an error, but got Ok"),
            None => (),
        }
    }
}
