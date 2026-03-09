use super::{
    runtime_find_game_path_with, runtime_get_library_dir_with, runtime_get_pulsar_root_with,
    runtime_log_with, snapshot_runtime_paths, AppRuntime,
};
use std::path::PathBuf;
use std::sync::Mutex;

struct FakeRuntime {
    game: Option<PathBuf>,
    library: Result<PathBuf, String>,
    root: Result<PathBuf, String>,
    logs: Mutex<Vec<(String, String)>>,
}

impl AppRuntime for FakeRuntime {
    fn find_game_path(&self) -> Option<PathBuf> {
        self.game.clone()
    }

    fn get_library_dir(&self) -> Result<PathBuf, String> {
        self.library.clone()
    }

    fn get_pulsar_root(&self) -> Result<PathBuf, String> {
        self.root.clone()
    }

    fn log(&self, level: &str, message: &str) {
        self.logs
            .lock()
            .expect("lock logs")
            .push((level.to_string(), message.to_string()));
    }
}

#[test]
fn snapshot_runtime_paths_reads_expected_values() {
    let runtime = FakeRuntime {
        game: Some(PathBuf::from("/game")),
        library: Ok(PathBuf::from("/library")),
        root: Ok(PathBuf::from("/pulsar")),
        logs: Mutex::new(Vec::new()),
    };

    let (game, library, root) = snapshot_runtime_paths(&runtime).expect("snapshot");
    assert_eq!(game, Some(PathBuf::from("/game")));
    assert_eq!(library, PathBuf::from("/library"));
    assert_eq!(root, PathBuf::from("/pulsar"));
}

#[test]
fn app_runtime_log_contract_is_callable() {
    let runtime = FakeRuntime {
        game: None,
        library: Ok(PathBuf::from("/library")),
        root: Ok(PathBuf::from("/pulsar")),
        logs: Mutex::new(Vec::new()),
    };
    runtime.log("INFO", "hello");
    let logs = runtime.logs.lock().expect("lock logs");
    assert_eq!(logs.as_slice(), [("INFO".to_string(), "hello".to_string())]);
}

#[test]
fn snapshot_runtime_paths_propagates_library_and_root_errors() {
    let library_err = FakeRuntime {
        game: Some(PathBuf::from("/game")),
        library: Err("library-failed".to_string()),
        root: Ok(PathBuf::from("/pulsar")),
        logs: Mutex::new(Vec::new()),
    };
    let err = snapshot_runtime_paths(&library_err).expect_err("expected library error");
    assert_eq!(err, "library-failed");

    let root_err = FakeRuntime {
        game: Some(PathBuf::from("/game")),
        library: Ok(PathBuf::from("/library")),
        root: Err("root-failed".to_string()),
        logs: Mutex::new(Vec::new()),
    };
    let err = snapshot_runtime_paths(&root_err).expect_err("expected root error");
    assert_eq!(err, "root-failed");
}

#[test]
fn runtime_helpers_forward_and_propagate() {
    let game = runtime_find_game_path_with(|| Some(PathBuf::from("/game")));
    assert_eq!(game, Some(PathBuf::from("/game")));
    assert_eq!(runtime_find_game_path_with(|| None), None);

    let library = runtime_get_library_dir_with(|| Ok(PathBuf::from("/library")))
        .expect("library helper should forward success");
    assert_eq!(library, PathBuf::from("/library"));
    let library_err = runtime_get_library_dir_with(|| Err("lib-failed".to_string()))
        .expect_err("library helper should propagate errors");
    assert_eq!(library_err, "lib-failed");

    let root = runtime_get_pulsar_root_with(|| Ok(PathBuf::from("/pulsar")))
        .expect("root helper should forward success");
    assert_eq!(root, PathBuf::from("/pulsar"));
    let root_err = runtime_get_pulsar_root_with(|| Err("root-failed".to_string()))
        .expect_err("root helper should propagate errors");
    assert_eq!(root_err, "root-failed");

    let logged = Mutex::new(Vec::<(String, String)>::new());
    runtime_log_with(
        |level, message| {
            logged
                .lock()
                .expect("log lock")
                .push((level.to_string(), message.to_string()))
        },
        "INFO",
        "hello",
    );
    assert_eq!(
        logged.lock().expect("log lock").as_slice(),
        [("INFO".to_string(), "hello".to_string())]
    );
}
