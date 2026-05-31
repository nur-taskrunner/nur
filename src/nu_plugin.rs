mod args;
mod commands;
mod engine;
mod errors;
mod names;
mod nu_version;
mod path;
mod scripts;
mod state;

use miette::Result;
use nu_protocol::engine::Command;
use nu_protocol::{
    Category, LabeledError, PipelineData, Signature, Span, SyntaxShape, Value
};
use std::env;
use std::path::PathBuf;

use nu_plugin::{EngineInterface, Plugin, PluginCommand};
use nu_plugin::{EvaluatedCall, MsgPackSerializer, serve_plugin};

use crate::commands::Nur;
use crate::engine::NurEngine;
use crate::errors::NurError;
use crate::names::NUR_FILE;

struct NurPlugin;

impl Plugin for NurPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(NurPluginCommand),
        ]
    }
}

struct NurPluginCommand;

impl PluginCommand for NurPluginCommand {
    type Plugin = NurPlugin;

    fn name(&self) -> &str {
        (Nur{}).name()
    }

    fn description(&self) -> &str {
        (Nur{}).description()
    }

    fn signature(&self) -> Signature {
        (Nur{}).signature()
    }

    fn run(
        &self,
        plugin: &Self::Plugin,
        engine: &EngineInterface,
        call: &EvaluatedCall,
        input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let engine = engine.clone();
        let run_path = PathBuf::from(engine.get_current_dir()?);
        let mut nur_engine = NurEngine::new_for_plugin(run_path, call)?;

        if !nur_engine.state.has_project_path {
            if nur_engine.state.nur_args.show_help {
                let help_string = nur_engine.get_help(&Nur);

                return Ok(PipelineData::Value(Value::string(help_string, Span::unknown()), None));
            } else {
                match nur_engine.state.nur_args.nurfile_name.clone() {
                    None => {
                        return Err(LabeledError::from(NurError::NurfileNotFound(
                            String::from(NUR_FILE),
                        )));
                    }
                    Some(Value::String {
                        val: nurfile_name, ..
                    }) => {
                        return Err(LabeledError::from(NurError::NurfileNotFound(
                            nurfile_name,
                        )));
                    }
                    Some(_) => {
                        return Err(LabeledError::from(NurError::NurfileNotFound(
                            String::from(NUR_FILE),
                        )));
                    }
                }
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
                return Ok(nur_engine.eval_for_plugin(
                    r#"scope commands
                    | where name starts-with "nur " and type == "custom"
                    | get name
                    | each { |it| $it | str substring 4.. }
                    | sort
                    | each { |it| print $it };
                    null"#,
                    PipelineData::empty(),
                )?);
            } else {
                return Ok(nur_engine.eval_for_plugin(
                    r#"scope commands
                    | where name starts-with "nur " and type == "custom"
                    | select name description
                    | update name { |row| $row.name | str substring 4.. }
                    | sort-by name
                    | table --index false"#,
                    PipelineData::empty(),
                )?);
            }
        }

        let run_command = if nur_engine.state.nur_args.run_commands.is_some() {
            nur_engine.state.nur_args.run_commands.clone().unwrap().item
        } else {
            nur_engine.state.task_call.join(" ")
        };
        let result = nur_engine.eval_for_plugin(run_command, input)?;

        Ok(result)
    }
}

impl From<NurError> for LabeledError {
    fn from(value: NurError) -> LabeledError {
        LabeledError::new(value.to_string())
    }
}

impl From<Box<NurError>> for LabeledError {
    fn from(value: Box<NurError>) -> LabeledError {
        LabeledError::new(value.to_string())
    }
}

fn main() {
    serve_plugin(&NurPlugin, MsgPackSerializer)
}
