use tempfile::tempdir;
use git2::Repository;

#[test]
fn test_repo_open_fail_non_git() {
    let dir = tempdir().unwrap();
    let res = Repository::open(dir.path());
    assert!(res.is_err());
}

#[test]
fn test_repo_init_and_open() {
    let dir = tempdir().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    assert!(repo.is_empty().unwrap());
}
