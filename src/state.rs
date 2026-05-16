use nu_protocol::Value;
use nu_protocol::engine::EngineState;

use crate::args::{NurArgs, gather_commandline_args, parse_commandline_args};
use crate::errors::{NurError, NurResult};
use crate::names::{
    NUR_CONFIG_CONFIG_FILENAME, NUR_CONFIG_DIR, NUR_CONFIG_ENV_FILENAME, NUR_CONFIG_LIB_PATH,
    NUR_FILE, NUR_FILE_DOT_NU, NUR_LOCAL_FILE, NUR_LOCAL_FILE_DOT_NU,
};
use crate::path::{find_nurfile, find_project_path};
use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct NurState {
    pub(crate) run_path: PathBuf,
    pub(crate) has_project_path: bool,
    pub(crate) project_path: PathBuf,

    pub(crate) config_dir: PathBuf,
    pub(crate) lib_dir_path: PathBuf,
    pub(crate) env_path: PathBuf,
    pub(crate) config_path: PathBuf,

    pub(crate) nurfile_path: Option<PathBuf>,
    pub(crate) local_nurfile_path: Option<PathBuf>,

    pub(crate) nur_args: NurArgs,
    pub(crate) has_task_call: bool,
    pub(crate) task_call: Vec<String>,
    pub(crate) task_name: Option<String>, // full task name, like "nur some-task"
}

impl NurState {
    pub(crate) fn new(
        engine_state: &mut EngineState,
        run_path: PathBuf,
        args: Vec<String>,
    ) -> NurResult<Self> {
        // Parse args into bits
        let cli_args = gather_commandline_args(args)?;
        let nur_args = parse_commandline_args(&cli_args.nur_args.join(" "), engine_state)?;

        // Define nurfile names
        let nurfile_names: Vec<String>;
        let nurfile_local_names: Vec<String>;
        match nur_args.nurfile_name.clone() {
            None => {
                nurfile_names = vec![String::from(NUR_FILE), String::from(NUR_FILE_DOT_NU)];
                nurfile_local_names = vec![
                    String::from(NUR_LOCAL_FILE),
                    String::from(NUR_LOCAL_FILE_DOT_NU),
                ];
            }
            Some(Value::String {
                val: nurfile_name, ..
            }) => {
                nurfile_names = vec![nurfile_name.clone()];
                if nurfile_name.ends_with(".nu") {
                    let nurfile_basename = nurfile_name.strip_suffix(".nu").unwrap();
                    nurfile_local_names = vec![format!("{nurfile_basename}.local.nu")];
                } else {
                    nurfile_local_names = vec![format!("{nurfile_name}.local")];
                }
            }
            Some(_) => {
                return Err(Box::new(NurError::InvalidNurfile()));
            }
        }

        // Get initial directory details
        let found_project_path = find_project_path(&run_path, &nurfile_names);
        let has_project_path = found_project_path.is_some();
        let project_path = found_project_path.unwrap_or(run_path.clone());

        // Set all paths
        let config_dir = project_path.join(NUR_CONFIG_DIR);
        let lib_dir_path = config_dir.join(NUR_CONFIG_LIB_PATH);
        let env_path = config_dir.join(NUR_CONFIG_ENV_FILENAME);
        let config_path = config_dir.join(NUR_CONFIG_CONFIG_FILENAME);

        // Set nurfiles
        let nurfile_path = find_nurfile(&project_path, &nurfile_names);
        let local_nurfile_path = find_nurfile(&project_path, &nurfile_local_names);

        Ok(NurState {
            run_path,
            has_project_path,
            project_path,

            config_dir,
            lib_dir_path,
            env_path,
            config_path,

            nurfile_path,
            local_nurfile_path,

            nur_args,
            has_task_call: cli_args.has_task_call,
            task_call: cli_args.task_call,
            task_name: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        engine::init_engine_state,
        names::{NUR_FILE, NUR_FILE_DOT_NU, NUR_LOCAL_FILE, NUR_LOCAL_FILE_DOT_NU},
    };

    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_nur_state_with_project_path() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let nurfile_path = temp_dir.path().join(NUR_FILE);
        File::create(&nurfile_path).unwrap();
        let nurfile_local_path = temp_dir.path().join(NUR_LOCAL_FILE);
        File::create(&nurfile_local_path).unwrap();

        // Setup test
        let args = vec![
            String::from("nur"),
            String::from("--quiet"),
            String::from("some_task"),
            String::from("task_arg"),
        ];
        let mut engine_state = init_engine_state(temp_dir_path.clone()).unwrap();
        let state = NurState::new(&mut engine_state, temp_dir_path.clone(), args).unwrap();

        // Check everything works out
        assert_eq!(state.run_path, temp_dir_path);
        assert_eq!(state.project_path, temp_dir_path);
        assert_eq!(state.has_project_path, true);

        assert_eq!(state.config_dir, temp_dir_path.join(".nur"));
        assert_eq!(state.lib_dir_path, temp_dir_path.join(".nur/scripts"));
        assert_eq!(state.env_path, temp_dir_path.join(".nur/env.nu"));
        assert_eq!(state.config_path, temp_dir_path.join(".nur/config.nu"));

        assert_eq!(state.nurfile_path.unwrap(), temp_dir_path.join(NUR_FILE),);
        assert_eq!(
            state.local_nurfile_path.unwrap(),
            temp_dir_path.join(NUR_LOCAL_FILE)
        );

        assert_eq!(state.nur_args.quiet_execution, true);
        assert_eq!(state.has_task_call, true);
        assert_eq!(
            state.task_call,
            vec![
                String::from("nur"),
                String::from("some_task"),
                String::from("task_arg")
            ]
        );

        // Clean up
        std::fs::remove_file(nurfile_path).unwrap();
    }

    #[test]
    fn test_nur_state_dot_nu_files() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();
        let nurfile_path = temp_dir.path().join(NUR_FILE_DOT_NU);
        File::create(&nurfile_path).unwrap();
        let nurfile_local_path = temp_dir.path().join(NUR_LOCAL_FILE_DOT_NU);
        File::create(&nurfile_local_path).unwrap();

        // Setup test
        let args = vec![
            String::from("nur"),
            String::from("--quiet"),
            String::from("some_task"),
            String::from("task_arg"),
        ];
        let mut engine_state = init_engine_state(temp_dir_path.clone()).unwrap();
        let state = NurState::new(&mut engine_state, temp_dir_path.clone(), args).unwrap();

        // Check nurfile paths are ok
        assert_eq!(
            state.nurfile_path.unwrap(),
            temp_dir_path.join(NUR_FILE_DOT_NU),
        );
        assert_eq!(
            state.local_nurfile_path.unwrap(),
            temp_dir_path.join(NUR_LOCAL_FILE_DOT_NU)
        );

        // Clean up
        std::fs::remove_file(nurfile_path).unwrap();
    }

    #[test]
    fn test_nur_state_without_project_path() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();

        // Setup test
        let args = vec![
            String::from("nur"),
            String::from("--quiet"),
            String::from("some_task"),
            String::from("task_arg"),
        ];
        let mut engine_state = init_engine_state(temp_dir_path.clone()).unwrap();
        let state = NurState::new(&mut engine_state, temp_dir_path.clone(), args).unwrap();

        // Check everything works out
        assert_eq!(state.run_path, temp_dir_path);
        assert_eq!(state.project_path, temp_dir_path); // same as run_path, as this is the fallback
        assert_eq!(state.has_project_path, false);

        assert_eq!(state.config_dir, temp_dir_path.join(".nur"));
        assert_eq!(state.lib_dir_path, temp_dir_path.join(".nur/scripts"));
        assert_eq!(state.env_path, temp_dir_path.join(".nur/env.nu"));
        assert_eq!(state.config_path, temp_dir_path.join(".nur/config.nu"));

        assert!(state.nurfile_path.is_none());
        assert!(state.local_nurfile_path.is_none());

        assert_eq!(state.nur_args.quiet_execution, true);
        assert_eq!(state.has_task_call, true);
        assert_eq!(
            state.task_call,
            vec![
                String::from("nur"),
                String::from("some_task"),
                String::from("task_arg")
            ]
        );
    }

    #[test]
    fn test_nur_state_without_task() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();

        // Setup test
        let args = vec![String::from("nur"), String::from("--help")];
        let mut engine_state = init_engine_state(temp_dir_path.clone()).unwrap();
        let state = NurState::new(&mut engine_state, temp_dir_path.clone(), args).unwrap();

        // Check everything works out
        assert_eq!(state.run_path, temp_dir_path);
        assert_eq!(state.project_path, temp_dir_path); // same as run_path, as this is the fallback
        assert_eq!(state.has_project_path, false);

        assert_eq!(state.config_dir, temp_dir_path.join(".nur"));
        assert_eq!(state.lib_dir_path, temp_dir_path.join(".nur/scripts"));
        assert_eq!(state.env_path, temp_dir_path.join(".nur/env.nu"));
        assert_eq!(state.config_path, temp_dir_path.join(".nur/config.nu"));

        assert!(state.nurfile_path.is_none());
        assert!(state.local_nurfile_path.is_none());

        assert_eq!(state.nur_args.show_help, true);
        assert_eq!(state.has_task_call, false);
        assert_eq!(state.task_call, vec![] as Vec<String>);
    }

    #[test]
    fn test_nur_state_with_nurfile_option() {
        let temp_dir = tempdir().unwrap();
        let temp_dir_path = temp_dir.path().to_path_buf();

        // Setup test
        let args = vec![String::from("nur"), String::from("--nurfile=other-nurfile")];
        let mut engine_state = init_engine_state(temp_dir_path.clone()).unwrap();
        let state = NurState::new(&mut engine_state, temp_dir_path.clone(), args).unwrap();

        // Check nurfile name is set
        assert_eq!(
            state.nur_args.nurfile_name.unwrap().as_str().unwrap(),
            "other-nurfile"
        );
    }
}
