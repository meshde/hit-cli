use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

thread_local! {
    static TEST_CONFIG_PATH: RefCell<Option<String>> = RefCell::new(None);
}

pub fn set_test_config_path(path: String) {
    TEST_CONFIG_PATH.with(|p| *p.borrow_mut() = Some(path));
}

pub fn get_test_config_path() -> Option<String> {
    TEST_CONFIG_PATH.with(|p| p.borrow().clone())
}

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join(".hit").join("config.json");
        
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        
        let test_config = serde_json::json!({
            "envs": {
                "dev": {
                    "API_URL": "https://dev-api.example.com"
                },
                "staging": {
                    "API_URL": "https://staging-api.example.com"
                }
            }
        });

        fs::write(&config_path, test_config.to_string()).unwrap();
        set_test_config_path(config_path.to_string_lossy().to_string());
        
        Self {
            temp_dir,
            config_path,
        }
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        TEST_CONFIG_PATH.with(|p| *p.borrow_mut() = None);
    }
}