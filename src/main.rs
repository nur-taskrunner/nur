mod args;
mod commands;
mod compat;
mod engine;
mod errors;
mod names;
mod nu_version;
mod path;
mod scripts;
mod state;

use crate::commands::Nur;
use crate::compat::show_nurscripts_hint;
use crate::engine::init_engine_state;
use crate::engine::NurEngine;
use crate::errors::NurError;
use crate::path::current_dir_from_environment;
use crate::state::NurState;
use miette::Result;
use nu_ansi_term::Color;
use nu_protocol::{ByteStream, PipelineData, Span};
use std::env;
use std::process::ExitCode;

fn main() -> Result<ExitCode, miette::ErrReport> {
    // Initialise nur state
    let run_path = current_dir_from_environment();
    let nur_state = NurState::new(run_path, env::args().collect())?;

    // Create raw nu engine state
    let engine_state = init_engine_state(&nur_state.project_path)?;

    // Setup nur engine from engine state
    let mut nur_engine = NurEngine::new(engine_state, nur_state)?;
    let use_color = nur_engine
        .engine_state
        .get_config()
        .use_ansi_coloring
        .get(&nur_engine.engine_state);

    // Parse args
    let parsed_nur_args = nur_engine.parse_args();

    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("run path: {:?}", nur_engine.state.run_path);
        eprintln!("project path: {:?}", nur_engine.state.project_path);
        eprintln!();
        eprintln!("nur args: {:?}", parsed_nur_args);
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

    // Show hints for compatibility issues
    if nur_engine.state.has_project_path {
        show_nurscripts_hint(nur_engine.state.project_path.clone(), use_color);
    }

    // Handle execution without project path, only allow to show help, abort otherwise
    if !nur_engine.state.has_project_path {
        if parsed_nur_args.show_help {
            nur_engine.print_help(&Nur);

            std::process::exit(0);
        } else {
            return Err(miette::ErrReport::from(NurError::NurfileNotFound()));
        }
    }

    // Load env and config
    nur_engine.load_env()?;
    nur_engine.load_config()?;

    // Load task files
    nur_engine.load_nurfiles()?;

    // Handle list tasks
    if parsed_nur_args.list_tasks {
        // TODO: Parse and handle commands without eval
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

        std::process::exit(0);
    }

    // Show help if no task call was found
    // (error exit if --help was not passed)
    if !nur_engine.state.has_task_call
        && parsed_nur_args.run_commands.is_none()
        && !parsed_nur_args.enter_shell
    {
        nur_engine.print_help(&Nur);
        if parsed_nur_args.show_help {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
    }

    // Handle help
    if parsed_nur_args.show_help {
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
    if nur_engine.state.has_task_call && parsed_nur_args.run_commands.is_some() {
        return Err(miette::ErrReport::from(NurError::InvalidNurCall(
            String::from("task call"),
            String::from("--commands/-c"),
        )));
    }
    if nur_engine.state.has_task_call && parsed_nur_args.enter_shell {
        return Err(miette::ErrReport::from(NurError::InvalidNurCall(
            String::from("task call"),
            String::from("--enter-shell"),
        )));
    }
    if parsed_nur_args.run_commands.is_some() && parsed_nur_args.enter_shell {
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
    let input = if parsed_nur_args.attach_stdin {
        PipelineData::ByteStream(ByteStream::stdin(Span::unknown())?, None)
    } else {
        PipelineData::empty()
    };

    // Execute the task
    let exit_code: i32;
    let run_command = if parsed_nur_args.run_commands.is_some() {
        parsed_nur_args.run_commands.clone().unwrap().item
    } else {
        nur_engine.state.task_call.join(" ")
    };
    #[cfg(feature = "debug")]
    if parsed_nur_args.debug_output {
        eprintln!("full command call: {}", run_command);
    }
    if parsed_nur_args.enter_shell {
        exit_code = match nur_engine.run_repl() {
            Ok(_) => 0,
            Err(_) => 1,
        }
    } else if parsed_nur_args.quiet_execution {
        exit_code = nur_engine.eval_and_print(run_command, input)?;

        #[cfg(feature = "debug")]
        if parsed_nur_args.debug_output {
            println!("Exit code {:?}", exit_code);
        }
    } else {
        println!("nur version {}", env!("CARGO_PKG_VERSION"));
        println!(
            "Project path: {}",
            nur_engine.state.project_path.to_str().unwrap()
        );
        if parsed_nur_args.run_commands.is_some() {
            println!("Running command: {}", run_command);
        } else {
            println!("Executing task: {}", nur_engine.get_short_task_name());
        }
        println!();
        exit_code = nur_engine.eval_and_print(run_command, input)?;
        #[cfg(feature = "debug")]
        if parsed_nur_args.debug_output {
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
