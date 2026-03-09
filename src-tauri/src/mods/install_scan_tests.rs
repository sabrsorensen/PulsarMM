use super::{
    add_unique_op, build_deploy_ops, folder_name_for_path, push_unique_op,
    scan_for_installable_mods, select_items_to_process, DeployOp,
};
#[cfg(unix)]
use std::ffi::OsString;
use std::fs;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn temp_test_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pulsarmm_{}_{}", prefix, Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("failed to create test temp dir");
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    fs::write(path, content).expect("failed to write test file");
}

#[test]
fn scan_marks_root_when_game_structure_folder_exists() {
    let root = temp_test_dir("scan_root");
    fs::create_dir_all(root.join("AUDIO")).expect("failed to create AUDIO folder");

    let found = scan_for_installable_mods(&root, &root);
    assert_eq!(found, vec![".".to_string()]);

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn scan_finds_nested_candidates_as_relative_paths() {
    let root = temp_test_dir("scan_nested");
    fs::create_dir_all(root.join("bundle/mod_a/METADATA")).expect("failed to create METADATA");
    fs::create_dir_all(root.join("bundle/mod_b/UI")).expect("failed to create UI");

    let mut found = scan_for_installable_mods(&root, &root);
    found.sort();

    assert_eq!(
        found,
        vec!["bundle/mod_a".to_string(), "bundle/mod_b".to_string()]
    );

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn scan_detects_supported_game_file_extensions_case_insensitive() {
    let root = temp_test_dir("scan_extensions");
    write_file(&root.join("MyFile.MBIN"), "dummy");

    let found = scan_for_installable_mods(&root, &root);
    assert_eq!(found, vec![".".to_string()]);

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn scan_returns_empty_for_irrelevant_tree() {
    let root = temp_test_dir("scan_empty");
    write_file(&root.join("readme.txt"), "not a mod");
    write_file(&root.join("LICENSE"), "no extension");
    fs::create_dir_all(root.join("notes")).expect("failed to create notes folder");

    let found = scan_for_installable_mods(&root, &root);
    assert!(found.is_empty());

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn scan_returns_empty_when_read_dir_fails_or_strip_prefix_misses() {
    let missing = temp_test_dir("scan_missing").join("does_not_exist");
    let found = scan_for_installable_mods(&missing, &missing);
    assert!(
        found.is_empty(),
        "missing roots should not produce candidates"
    );

    let root = temp_test_dir("scan_strip_prefix");
    fs::create_dir_all(root.join("UI")).expect("failed to create UI folder");
    let unrelated_base = temp_test_dir("scan_unrelated_base");
    let found = scan_for_installable_mods(&root, &unrelated_base);
    assert!(
        found.is_empty(),
        "when base_dir is unrelated, relative path conversion should skip candidate"
    );

    fs::remove_dir_all(root).expect("failed to clean temp dir");
    fs::remove_dir_all(unrelated_base).expect("failed to clean temp dir");
}

#[test]
fn scan_stops_at_first_mod_root_and_does_not_descend() {
    let root = temp_test_dir("scan_stop");
    fs::create_dir_all(root.join("UI")).expect("failed to create UI folder");
    fs::create_dir_all(root.join("nested/METADATA")).expect("failed to create nested folder");

    let found = scan_for_installable_mods(&root, &root);
    assert_eq!(
        found,
        vec![".".to_string()],
        "root mod markers should short-circuit recursion"
    );

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[cfg(unix)]
#[test]
fn scan_handles_non_utf8_subdirectories() {
    let root = temp_test_dir("scan_non_utf8");
    let invalid = OsString::from_vec(vec![0x66, 0x6f, 0x80]);
    let mod_dir = root.join(PathBuf::from(invalid));
    fs::create_dir_all(mod_dir.join("UI")).expect("failed to create UI folder");

    let found = scan_for_installable_mods(&root, &root);
    assert_eq!(found.len(), 1);

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn select_items_uses_selected_folders_when_provided() {
    let root = temp_test_dir("select_items");
    fs::create_dir_all(root.join("A")).expect("failed to create A");
    fs::create_dir_all(root.join("B")).expect("failed to create B");

    let selected = vec!["B".to_string()];
    let items = select_items_to_process(&root, &selected).expect("selection should succeed");
    assert_eq!(items, vec!["B".to_string()]);

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn select_items_scans_top_level_when_empty_or_dot() {
    let root = temp_test_dir("select_scan");
    fs::create_dir_all(root.join("DirOne")).expect("failed to create DirOne");
    fs::create_dir_all(root.join("DirTwo")).expect("failed to create DirTwo");
    write_file(&root.join("file.txt"), "x");

    let mut items = select_items_to_process(&root, &[]).expect("selection should succeed");
    items.sort();
    assert_eq!(items, vec!["DirOne".to_string(), "DirTwo".to_string()]);

    let mut dot_items =
        select_items_to_process(&root, &[".".to_string()]).expect("dot selection should succeed");
    dot_items.sort();
    assert_eq!(dot_items, vec!["DirOne".to_string(), "DirTwo".to_string()]);

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn select_items_errors_when_source_root_cannot_be_read() {
    let root = temp_test_dir("select_err");
    let file = root.join("not_a_dir");
    write_file(&file, "x");

    let err = select_items_to_process(&file, &[]).expect_err("expected read_dir error");
    assert!(!err.is_empty());

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn build_ops_skips_missing_and_deduplicates_case_insensitive() {
    let root = temp_test_dir("build_ops_dedupe");
    fs::create_dir_all(root.join("Alpha")).expect("failed to create Alpha");
    fs::create_dir_all(root.join("alpha")).expect("failed to create alpha");

    let ops = build_deploy_ops(
        &root,
        vec![
            "Alpha".to_string(),
            "alpha".to_string(),
            "Missing".to_string(),
        ],
        false,
    )
    .expect("build_deploy_ops should succeed");

    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].dest_name.to_lowercase(), "alpha");
    assert!(ops[0].source.exists());

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn build_ops_flatten_falls_back_to_source_folder_when_no_nested_candidates() {
    let root = temp_test_dir("build_ops_flatten_fallback");
    fs::create_dir_all(root.join("Bundle/docs")).expect("failed to create docs folder");

    let ops = build_deploy_ops(&root, vec!["Bundle".to_string()], true)
        .expect("build_deploy_ops should succeed");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].dest_name, "Bundle");
    assert!(ops[0].source.ends_with("Bundle"));

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn build_ops_flatten_uses_root_candidate_when_scan_returns_dot() {
    let root = temp_test_dir("build_ops_flatten_dot");
    fs::create_dir_all(root.join("UI")).expect("failed to create UI folder");

    let ops =
        build_deploy_ops(&root, vec![".".to_string()], true).expect("build_deploy_ops should work");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].source, root);
    assert_eq!(
        ops[0].dest_name,
        ops[0]
            .source
            .file_name()
            .expect("temp dir should have a name")
            .to_string_lossy()
    );

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn build_ops_flatten_fallback_deduplicates_case_insensitive() {
    let root = temp_test_dir("build_ops_flatten_fallback_dedupe");
    fs::create_dir_all(root.join("Bundle/docs")).expect("failed to create docs folder");
    fs::create_dir_all(root.join("bundle/notes")).expect("failed to create notes folder");

    let ops = build_deploy_ops(
        &root,
        vec!["Bundle".to_string(), "bundle".to_string()],
        true,
    )
    .expect("build_deploy_ops should succeed");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].dest_name.to_lowercase(), "bundle");

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn build_ops_flattens_nested_installable_roots() {
    let root = temp_test_dir("build_ops_flatten");
    fs::create_dir_all(root.join("Bundle/OptionA/UI")).expect("failed to create OptionA");
    fs::create_dir_all(root.join("Bundle/OptionB/METADATA")).expect("failed to create OptionB");

    let mut names: Vec<String> = build_deploy_ops(&root, vec!["Bundle".to_string()], true)
        .expect("build_deploy_ops should succeed")
        .into_iter()
        .map(|op| op.dest_name)
        .collect();
    names.sort();

    assert_eq!(names, vec!["OptionA".to_string(), "OptionB".to_string()]);

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn build_ops_flatten_deduplicates_nested_targets_case_insensitive() {
    let root = temp_test_dir("build_ops_flatten_case");
    fs::create_dir_all(root.join("Bundle/OptionA/UI")).expect("failed to create OptionA");
    fs::create_dir_all(root.join("Bundle/optiona/METADATA")).expect("failed to create optiona");

    let ops = build_deploy_ops(&root, vec!["Bundle".to_string()], true)
        .expect("build_deploy_ops should succeed");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].dest_name.to_lowercase(), "optiona");

    fs::remove_dir_all(root).expect("failed to clean temp dir");
}

#[test]
fn build_ops_supports_dot_item_and_flattening_root() {
    let root = temp_test_dir("build_ops_dot");
    fs::create_dir_all(root.join("OptionA/UI")).expect("failed to create OptionA");

    let ops = build_deploy_ops(&root, vec![".".to_string()], false)
        .expect("dot deploy ops should succeed");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].source, root);
}

#[test]
fn build_ops_errors_when_root_name_is_missing() {
    let err = build_deploy_ops(Path::new("/"), vec![".".to_string()], false)
        .expect_err("root path should not have a folder name");
    assert_eq!(err, "Invalid path");
}

#[test]
fn folder_name_for_path_extracts_name_and_rejects_root() {
    assert_eq!(
        folder_name_for_path(Path::new("/tmp/Example")).expect("folder name should exist"),
        "Example"
    );
    let err = folder_name_for_path(Path::new("/")).expect_err("root path should be invalid");
    assert_eq!(err, "Invalid path");
}

#[test]
fn push_unique_op_deduplicates_case_insensitively() {
    let mut ops = vec![DeployOp {
        source: PathBuf::from("/tmp/A"),
        dest_name: "Alpha".to_string(),
    }];

    push_unique_op(&mut ops, PathBuf::from("/tmp/a"), "alpha".to_string());
    push_unique_op(&mut ops, PathBuf::from("/tmp/B"), "Beta".to_string());

    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].dest_name, "Alpha");
    assert_eq!(ops[1].dest_name, "Beta");
}

#[test]
fn add_unique_op_derives_name_and_rejects_invalid_root_paths() {
    let mut ops = Vec::new();
    add_unique_op(&mut ops, PathBuf::from("/tmp/Gamma")).expect("valid path should succeed");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].dest_name, "Gamma");

    let err = add_unique_op(&mut ops, PathBuf::from("/")).expect_err("root path should be invalid");
    assert_eq!(err, "Invalid path");
}
