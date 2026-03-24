use std::path::PathBuf;

pub fn parse_path(s: &str) -> Result<PathBuf, String> {
    Ok(PathBuf::from(shellexpand::tilde(s).as_ref()))
}
