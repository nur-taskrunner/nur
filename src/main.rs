mod args;
mod commands;
mod engine;
mod errors;
mod names;
mod nu_version;
mod path;
mod scripts;
mod state;

use crate::commands::Nur;
use crate::engine::NurEngine;
use crate::errors::NurError;
use crate::names::{NUR_FILE, NUR_QUIET};
use crate::path::current_dir_from_environment;
use miette::Result;
use nu_ansi_term::Color;
use nu_protocol::shell_error::generic::GenericError;
use nu_protocol::{ByteStream, PipelineData, ShellError, Span, Value};
use std::env;
use std::process::ExitCode;

fn main() -> Result<ExitCode, miette::ErrReport> {
    // Initialise nur engine with current path
    let run_path = current_dir_from_environment();
    let mut nur_engine = NurEngine::new(run_path, env::args().collect())?;

    let use_color = nur_engine
        .engine_state
        .get_config()
        .use_ansi_coloring
        .get(&nur_engine.engine_state);

    #[cfg(feature = "debug")]
    if nur_engine.state.nur_args.debug_output {
        eprintln!("run path: {:?}", nur_engine.state.run_path);
        eprintln!("project path: {:?}", nur_engine.state.project_path);
        eprintln!();
        eprintln!("nur args: {:?}", nur_engine.state.nur_args);
        eprintln!("task call: {:?}", nur_engine.state.task_call);
        eprintln!();
        eprintln!("nur config dir: {:?}", nur_engine.state.config_dir);
        eprintln!(
            "nur lib path (scripts/): {:?}",
            nur_engine.state.lib_dir_path
        );
        eprintln!("nur env path (env.nu): {:?}", nur_engine.state.env_path);
        eprintln!(
            "nur config path (config.nu): {:?}",
            nur_engine.state.config_path
        );
        eprintln!();
        eprintln!("nurfile path: {:?}", nur_engine.state.nurfile_path);
        eprintln!(
            "nurfile local path: {:?}",
            nur_engine.state.local_nurfile_path
        );
    }

    // Enable ctrl+c protection (ensures clean exit when running background jobs)
    nur_engine.ctrlc_protection();

    // Handle execution without project path, only allow to show help, abort otherwise
    if !nur_engine.state.has_project_path {
        if nur_engine.state.nur_args.show_help {
            nur_engine.print_help(&Nur);

            std::process::exit(0);
        } else {
            match nur_engine.state.nur_args.nurfile_name.clone() {
                None => {
                    return Err(miette::ErrReport::from(NurError::NurfileNotFound(
                        String::from(NUR_FILE),
                    )));
                }
                Some(Value::String {
                    val: nurfile_name, ..
                }) => {
                    return Err(miette::ErrReport::from(NurError::NurfileNotFound(
                        nurfile_name,
                    )));
                }
                Some(_) => {
                    return Err(miette::ErrReport::from(NurError::NurfileNotFound(
                        String::from(NUR_FILE),
                    )));
                }
            }
        }
    }

    // Load .env file from project directory - if requested
    match &nur_engine.state.nur_args.dotenv {
        None => {
            let env_path = nur_engine.state.project_path.join(".env");

            if env_path.exists() && !env_path.is_dir() {
                nur_engine.load_dot_env(env_path)?;

                // If we now have NUR_QUIET set, update parsed args state
                if nur_engine.engine_state.get_env_var(NUR_QUIET).is_some() {
                    nur_engine.state.nur_args.quiet_execution = true;
                }
            }
        }
        Some(Value::String { val, .. }) => {
            let env_path = nur_engine.state.project_path.join(val);
            if !env_path.exists() {
                return Err(miette::ErrReport::from(NurError::DotenvFileError(
                    val.into(),
                    String::from("dotenv file does not exist"),
                )));
            }
            if env_path.is_dir() {
                return Err(miette::ErrReport::from(NurError::DotenvFileError(
                    val.into(),
                    String::from("dotenv file is actually a directory"),
                )));
            }

            nur_engine.load_dot_env(env_path)?;

            // If we now have NUR_QUIET set, update parsed args state
            if nur_engine.engine_state.get_env_var(NUR_QUIET).is_some() {
                nur_engine.state.nur_args.quiet_execution = true;
            }
        }
        Some(Value::Nothing { .. }) => {} // nothing to do
        Some(_) => {
            return Err(miette::ErrReport::from(ShellError::Generic(
                GenericError::new_internal(
                    "--dotenv must either be null (do not load .env) or a filepath",
                    "",
                ),
            )));
        }
    }

    // Load env and config
    nur_engine.load_env()?;
    nur_engine.load_config()?;

    // Load autoload files
    nur_engine.read_autoload_files()?;

    // Load task files
    nur_engine.load_nurfiles()?;

    // Handle list tasks
    if nur_engine.state.nur_args.list_tasks {
        // TODO: Parse and handle commands without eval
        if nur_engine.state.nur_args.quiet_execution {
            nur_engine.eval_and_print(
                r#"scope commands
                | where name starts-with "nur " and type == "custom"
                | get name
                | each { |it| $it | str substring 4.. }
                | sort
                | each { |it| print $it };
                null"#,
                PipelineData::empty(),
            )?;
        } else {
            println!("nur version {}", env!("CARGO_PKG_VERSION"));
            println!(
                "Project path: {}",
                nur_engine.state.project_path.to_str().unwrap()
            );
            println!();
            nur_engine.eval_and_print(
                r#"scope commands
                | where name starts-with "nur " and type == "custom"
                | select name description
                | update name { |row| $row.name | str substring 4.. }
                | sort-by name
                | table --index false"#,
                PipelineData::empty(),
            )?;
        }

        std::process::exit(0);
    }

    // Show help if no task call was found
    // (error exit if --help was not passed)
    if !nur_engine.state.has_task_call
        && nur_engine.state.nur_args.run_commands.is_none()
        && !nur_engine.state.nur_args.enter_shell
    {
        nur_engine.print_help(&Nur);
        if nur_engine.state.nur_args.show_help {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    }

    // Handle help
    if nur_engine.state.nur_args.show_help {
        if !nur_engine.state.has_task_call {
            nur_engine.print_help(&Nur);
            std::process::exit(0);
        }

        if let Some(command) = nur_engine.clone().get_task_def() {
            nur_engine.clone().print_help(command);
            std::process::exit(0);
        } else {
            return Err(miette::ErrReport::from(NurError::TaskNotFound(
                nur_engine.state.task_call.join(" "),
            )));
        }
    }

    // Ensure we only allow sane calls
    if nur_engine.state.has_task_call && nur_engine.state.nur_args.run_commands.is_some() {
        return Err(miette::ErrReport::from(NurError::InvalidNurCall(
            String::from("task call"),
            String::from("--commands/-c"),
        )));
    }
    if nur_engine.state.has_task_call && nur_engine.state.nur_args.enter_shell {
        return Err(miette::ErrReport::from(NurError::InvalidNurCall(
            String::from("task call"),
            String::from("--enter-shell"),
        )));
    }
    if nur_engine.state.nur_args.run_commands.is_some() && nur_engine.state.nur_args.enter_shell {
        return Err(miette::ErrReport::from(NurError::InvalidNurCall(
            String::from("--commands/-c"),
            String::from("--enter-shell"),
        )));
    }
    if nur_engine.state.has_task_call && nur_engine.state.task_name.is_none() {
        return Err(miette::ErrReport::from(NurError::TaskNotFound(
            nur_engine.state.task_call.join(" "),
        )));
    }

    // Prepare input data - if requested
    let input = if nur_engine.state.nur_args.attach_stdin {
        PipelineData::ByteStream(ByteStream::stdin(Span::unknown())?, None)
    } else {
        PipelineData::empty()
    };

    // Execute the task
    let exit_code: i32;
    let run_command = if nur_engine.state.nur_args.run_commands.is_some() {
        nur_engine.state.nur_args.run_commands.clone().unwrap().item
    } else {
        nur_engine.state.task_call.join(" ")
    };
    #[cfg(feature = "debug")]
    if nur_engine.state.nur_args.debug_output {
        eprintln!("full command call: {}", run_command);
    }
    if nur_engine.state.nur_args.enter_shell {
        if !nur_engine.state.nur_args.quiet_execution {
            println!("nur version {}", env!("CARGO_PKG_VERSION"));
            println!(
                "Project path: {}",
                nur_engine.state.project_path.to_str().unwrap()
            );
            println!("Entering repl shell...");
            println!();
        }
        exit_code = match nur_engine.run_repl() {
            Ok(_) => 0,
            Err(_) => 1,
        }
    } else if nur_engine.state.nur_args.quiet_execution {
        exit_code = nur_engine.eval_and_print(run_command, input)?;

        #[cfg(feature = "debug")]
        if nur_engine.state.nur_args.debug_output {
            println!("Exit code {:?}", exit_code);
        }
    } else {
        println!("nur version {}", env!("CARGO_PKG_VERSION"));
        println!(
            "Project path: {}",
            nur_engine.state.project_path.to_str().unwrap()
        );
        if nur_engine.state.nur_args.run_commands.is_some() {
            println!("Running command: {run_command}");
        } else {
            println!("Executing task: {}", nur_engine.get_short_task_name());
        }
        println!();
        exit_code = nur_engine.eval_and_print(run_command, input)?;
        #[cfg(feature = "debug")]
        if nur_engine.state.nur_args.debug_output {
            println!("Exit code {:?}", exit_code);
        }
        if exit_code == 0 {
            println!(
                "{}Task execution successful{}",
                if use_color {
                    Color::Green.prefix().to_string()
                } else {
                    String::from("")
                },
                if use_color {
                    Color::Green.suffix().to_string()
                } else {
                    String::from("")
                },
            );
        } else {
            println!(
                "{}Task execution failed (exit code: {}){}",
                if use_color {
                    Color::Red.prefix().to_string()
                } else {
                    String::from("")
                },
                exit_code,
                if use_color {
                    Color::Red.suffix().to_string()
                } else {
                    String::from("")
                },
            );
        }
    }

    Ok(ExitCode::from(exit_code as u8))
}
