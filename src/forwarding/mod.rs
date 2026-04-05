use anyhow::Result;
use std::process::Command;

pub fn forward_to_git(args: Vec<String>) -> Result<()> {
    let mut child = Command::new("git").args(args).spawn()?;

    let status = child.wait()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
