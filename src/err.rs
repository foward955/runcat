#[derive(Debug)]
pub(crate) enum RunCatTrayError {
    RunAppFailed(&'static str),
    FileError(&'static str),
    PathError(&'static str),
    Other(&'static str),
}
