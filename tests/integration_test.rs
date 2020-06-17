use moy_sekret::{init};
use std::fs;
use std::panic;
use std::path::Path;

// Fixtures
//

const F_KEYS_DIR: &str = "test_keys";
const F_PROFILE: &str = "tester";

// Helpers
//

fn run_test<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    before_test();
    let result = panic::catch_unwind(|| test());
    after_test();

    assert!(result.is_ok())
}

#[allow(dead_code)]
fn before_test() {
    remove_keys_dir();
}

#[allow(dead_code)]
fn after_test() {
    remove_keys_dir();
}

#[allow(dead_code)]
fn create_keys_dir() {
    let keys_dir = Path::new(F_KEYS_DIR);
    match fs::create_dir_all(keys_dir) {
        Ok(_) => (),
        Err(reason) => assert!(
            false,
            format!("Failed to create test keys' directory: {}", reason)
        ),
    }
}

fn remove_keys_dir() {
    let keys_dir = Path::new(F_KEYS_DIR);
    let _ = fs::remove_dir_all(keys_dir);
}

fn check_key_file_exists(keys_dir: &String, profile: &String, key: &str) -> bool {
    let file_path = format!("{}/{}.{}", keys_dir, profile, key);
    let file = Path::new(file_path.as_str());
    file.is_file()
}

fn check_key_file_not_exists(keys_dir: &String, profile: &String, key: &str) -> bool {
    !check_key_file_exists(keys_dir, profile, key)
}

// Tests
//

#[test]
fn should_init_for_profile_and_save_them_to_a_given_directory() {
    run_test(|| {
        let keys_dir = F_KEYS_DIR.to_string();
        let profile = F_PROFILE.to_string();

        match init(&keys_dir, &profile) {
            Ok(_) => {
                assert!(
                    check_key_file_exists(&keys_dir, &profile, "pk"),
                    format!("Should exist a {}.pk file in {} dir", &profile, &keys_dir)
                );
                assert!(
                    check_key_file_exists(&keys_dir, &profile, "sk"),
                    format!("Should exist a {}.sk file in {} dir", &profile, &keys_dir)
                );
            }
            Err(e) => assert!(false, format!("Should have created but: {}", e)),
        }
    })
}

#[test]
fn should_not_init_due_to_permission_denied_on_keys_directory() {
    // Have to find out how to test it on Windows but not now
    if cfg!(windows) {
        assert!(true);
        return;
    }

    run_test(|| {
        let keys_dir = String::from("/keys");
        let profile = F_PROFILE.to_string();

        match init(&keys_dir, &profile) {
            Ok(_) => assert!(false, "Should not create key pair"),
            Err(e) => {
                assert_eq!(
                    "Initialization failed: Could not create keys dir: Failed to create keys' directory: Permission denied (os error 13)",
                    e.to_string()
                );
                assert!(
                    check_key_file_not_exists(&keys_dir, &profile, "pk"),
                    format!("Should not exist a {}.pk file in {} dir", &profile, &keys_dir)
                );
                assert!(
                    check_key_file_not_exists(&keys_dir, &profile, "sk"),
                    format!("Should not exist a {}.sk file in {} dir", &profile, &keys_dir)
                );
            }
        }
    })
}
