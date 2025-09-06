use nu_engine::get_full_help;
use nu_protocol::engine::{Command, EngineState, Stack};
use nu_protocol::{
    Category, Example, IntoPipelineData, PipelineData, ShellError, Signature, SyntaxShape, Value,
};

#[derive(Clone)]
pub(crate) struct Nur;

impl Command for Nur {
    fn name(&self) -> &str {
        "nur"
    }

    fn signature(&self) -> Signature {
        let mut signature = Signature::build("nur");

        signature = signature
            .description("nur - a taskrunner based on nu shell.")
            .switch("version", "Output version number and exit", Some('v'))
            .switch("list", "List available tasks and then just exit", Some('l'))
            .switch(
                "quiet",
                "Do not output anything but what the task produces",
                Some('q'),
            )
            .switch("stdin", "Attach stdin to called nur task", None)
            .named(
                "commands",
                SyntaxShape::String,
                "Run the given commands after nurfiles have been loaded",
                Some('c'),
            )
            .named(
                "dotenv",
                SyntaxShape::OneOf(vec![SyntaxShape::Nothing, SyntaxShape::Filepath]),
                "Load the dotenv file at the specified file path (default is to load .env in the project directory). Set to null if no dotenv file should be loaded",
                None,
            )
            .switch(
                "enter-shell",
                "Enter a nu REPL shell after the nurfiles have been loaded (use only for debugging)",
                None,
            )
            .optional(
                "task name",
                SyntaxShape::String,
                "Name of the task to run (you may use sub tasks)",
            )
            .rest(
                "task args",
                SyntaxShape::String,
                "Parameters for the executed task",
            )
            .category(Category::Default);

        #[cfg(feature = "debug")]
        {
            signature = signature.switch("debug", "Show debug details", Some('d'));
        }

        signature
    }

    fn description(&self) -> &str {
        "nur - a taskrunner based on nu shell."
    }

    fn run(
        &self,
        engine_state: &EngineState,
        stack: &mut Stack,
        call: &nu_protocol::engine::Call,
        _input: PipelineData,
    ) -> Result<PipelineData, ShellError> {
        Ok(Value::string(get_full_help(&Nur, engine_state, stack), call.head).into_pipeline_data())
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                description: "Execute a task",
                example: "nur task-name",
                result: None,
            },
            Example {
                description: "List available tasks",
                example: "nur --list",
                result: None,
            },
        ]
    }
}
