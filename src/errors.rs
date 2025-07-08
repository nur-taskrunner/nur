use miette::{Diagnostic, Report};
use nu_protocol::{ParseError, ShellError};
use thiserror::Error;

pub(crate) type NurResult<T> = Result<T, Box<NurError>>;

#[derive(Clone, Debug, Error, Diagnostic)]
pub enum NurError {
    #[error("Init nu error {0}")]
    #[diagnostic()]
    InitError(String),

    #[error("IO Error {0}")]
    #[diagnostic()]
    IoError(String),

    #[error("Shell Error {0}")]
    #[diagnostic()]
    ShellError(#[from] ShellError),

    #[error("Parse Errors")]
    #[diagnostic()]
    ParseErrors(#[related] Vec<ParseError>),

    #[error("Invalid task name '{0}'")]
    #[diagnostic()]
    InvalidTaskName(String),

    #[error("Could not find the task for call '{0}'")]
    #[diagnostic()]
    TaskNotFound(String),

    #[error("Could not find nurfile in path and parents")]
    #[diagnostic()]
    NurfileNotFound(),

    #[error("Entered shell did raise an error")]
    #[diagnostic()]
    EnteredShellError(),

    #[error("You cannot use {0} and {1} together")]
    #[diagnostic()]
    InvalidNurCall(String, String),

    #[error("Could not load dotenv file or not a file at {0}")]
    #[diagnostic()]
    DotenvFileError(String),

    #[error("Could not parse dotenv file: {0}")]
    #[diagnostic()]
    DotenvParseError(String),
}

impl From<std::io::Error> for Box<NurError> {
    fn from(_value: std::io::Error) -> Box<NurError> {
        Box::new(NurError::IoError(String::from("Could not read file")))
    }
}

impl From<ShellError> for Box<NurError> {
    fn from(_value: ShellError) -> Box<NurError> {
        Box::new(NurError::from(_value))
    }
}

impl From<Box<NurError>> for Report {
    fn from(err: Box<NurError>) -> Self {
        Report::new(*err) // move out of the Box
    }
}
