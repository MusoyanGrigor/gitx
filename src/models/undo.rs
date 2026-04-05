use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoPlan {
    pub staged_files: Vec<String>,
    pub unstaged_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub last_commit: Option<CommitUndoInfo>,
    pub actions: Vec<UndoAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitUndoInfo {
    pub hash: String,
    pub subject: String,
    pub already_pushed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UndoAction {
    Unstage(String),
    Discard(String),
    RemoveUntracked(String),
    ResetCommit { hash: String, mode: ResetMode },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}
