use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Grammar error: {0}")]
    Grammar(String),
    #[error("Unknown shape: {0}")]
    UnknownShape(String),
    #[error("Unknown edge route: {0}")]
    UnknownRoute(String),
    #[error("Unknown direction: {0}")]
    UnknownDirection(String),
    #[error("Unknown diagram type: {0}")]
    UnknownDiagramType(String),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
}
