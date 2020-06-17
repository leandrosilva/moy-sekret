use moy_sekret::{init, profile_exists};
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

// Tests
//

#[test]
fn should_init_a_profile_and_save_them_to_a_given_directory() {
    run_test(|| {
        let keys_dir = F_KEYS_DIR.to_string();
        let profile = F_PROFILE.to_string();

        match init(&keys_dir, &profile, false) {
            Ok(_) => {
                match profile_exists(&keys_dir, &profile) {
                Ok(_) => assert!(true),
                Err(reason) => assert!(
                    false,
                    format!("Should exist a key pair for {} in {} dir but: {}", &profile, &keys_dir, reason)
                )
            }
            },
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

        match init(&keys_dir, &profile, false) {
            Ok(_) => assert!(false, "Should not create key pair"),
            Err(e) => {
                assert_eq!(
                    "Initialization failed: Could not create keys dir: Failed to create keys' directory: Permission denied (os error 13)",
                    e.to_string()
                );
                match profile_exists(&keys_dir, &profile) {
                    Ok(_) => assert!(
                        false,
                        format!("Should not exist any key file for {} in {} dir", &profile, &keys_dir)
                    ),
                    Err(_) => assert!(true),
                };
            }
        }
    })
}
