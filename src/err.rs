#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum RunCatTrayError {
    RunAppFailed(String),
    FileError(String),
    PathError(String),
    Other(String),
}
