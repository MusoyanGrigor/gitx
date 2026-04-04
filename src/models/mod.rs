pub mod undo;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub date: i64, 
    pub subject: String,
    pub body: Option<String>,
    pub labels: Vec<LabelInfo>,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LabelInfo {
    Head(String),
    LocalBranch(String),
    RemoteBranch(String),
    Tag(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub base_hash: String,
    pub unique_to_a: Vec<CommitInfo>,
    pub unique_to_b: Vec<CommitInfo>,
}
