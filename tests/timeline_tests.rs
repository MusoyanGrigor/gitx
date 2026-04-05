use git2::{Repository, Signature};
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_timeline_cli_basic() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig = Signature::now("Test Author", "test@example.com").unwrap();

    // Create first commit
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
        .unwrap();

    // Run gitx timeline
    let gitx_bin = std::env::current_dir().unwrap().join("target/debug/gitx");
    let output = Command::new(gitx_bin)
        .arg("timeline")
        .arg("--limit")
        .arg("1")
        .current_dir(dir.path())
        .output()
        .expect("failed to execute gitx");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("initial commit"));
    assert!(stdout.contains("Test Author"));
}

#[test]
fn test_timeline_filter_author() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig1 = Signature::now("Author One", "one@example.com").unwrap();
    let sig2 = Signature::now("Author Two", "two@example.com").unwrap();

    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    let c1 = repo
        .commit(Some("HEAD"), &sig1, &sig1, "commit one", &tree, &[])
        .unwrap();
    let parent = repo.find_commit(c1).unwrap();
    repo.commit(Some("HEAD"), &sig2, &sig2, "commit two", &tree, &[&parent])
        .unwrap();

    let gitx_bin = std::env::current_dir().unwrap().join("target/debug/gitx");

    // Filter by Author One
    let output = Command::new(&gitx_bin)
        .arg("timeline")
        .arg("--author")
        .arg("One")
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("commit one"));
    assert!(!stdout.contains("commit two"));
}

#[test]
fn test_timeline_limit() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig = Signature::now("A", "a@b.com").unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    let mut parent_id = None;
    for i in 0..10 {
        let parents = if let Some(p) = parent_id {
            vec![repo.find_commit(p).unwrap()]
        } else {
            vec![]
        };
        let parents_ref: Vec<&git2::Commit> = parents.iter().collect();
        parent_id = Some(
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!("msg {}", i),
                &tree,
                &parents_ref,
            )
            .unwrap(),
        );
    }

    let gitx_bin = std::env::current_dir().unwrap().join("target/debug/gitx");
    let output = Command::new(gitx_bin)
        .arg("timeline")
        .arg("--limit")
        .arg("3")
        .current_dir(dir.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should see msg 9, msg 8, msg 7 (reverse chronological)
    assert!(stdout.contains("msg 9"));
    assert!(stdout.contains("msg 8"));
    assert!(stdout.contains("msg 7"));
    assert!(!stdout.contains("msg 6"));
}

#[test]
fn test_timeline_merges() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let sig = Signature::now("A", "a@b.com").unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    let c1 = repo
        .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
        .unwrap();
    let parent1 = repo.find_commit(c1).unwrap();

    // Create branch
    let c2 = repo
        .commit(None, &sig, &sig, "branch commit", &tree, &[&parent1])
        .unwrap();
    let branch_commit = repo.find_commit(c2).unwrap();

    // Merge commit
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "merge commit",
        &tree,
        &[&parent1, &branch_commit],
    )
    .unwrap();

    let gitx_bin = std::env::current_dir().unwrap().join("target/debug/gitx");

    // Test --merges
    let output = Command::new(&gitx_bin)
        .arg("timeline")
        .arg("--merges")
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("merge commit"));
    assert!(!stdout.contains("branch commit"));

    // Test --no-merges
    let output = Command::new(&gitx_bin)
        .arg("timeline")
        .arg("--no-merges")
        .current_dir(dir.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("merge commit"));
    assert!(stdout.contains("branch commit"));
}
