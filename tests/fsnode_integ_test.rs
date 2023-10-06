use light_server::FsNode;

fn is_dir(x: &FsNode) -> bool {
    match x {
        FsNode::Dir(_) => true,
        _ => false,
    }
}

fn is_file(x: &FsNode) -> bool {
    match x {
        FsNode::File(_) => true,
        _ => false,
    }
}

#[test]
fn it_processes_files_from_fspath() {
    let result = FsNode::from_fs(".");
    assert!(
        result.is_ok() && is_dir(result.as_ref().unwrap()),
        ". not a dir"
    );
    let dir_map = match result.unwrap() {
        FsNode::Dir(dir_map) => Ok(dir_map),
        _ => Err(()),
    }
    .unwrap();
    assert!(dir_map.contains_key("Cargo.toml"), "no Cargo.toml");
    assert!(
        is_file(dir_map.get("Cargo.toml").unwrap()),
        "Cargo.toml not a file"
    );
    assert!(dir_map.contains_key("src"), "no src directory");
    assert!(is_dir(dir_map.get("src").unwrap()), "src not a directory");
}
