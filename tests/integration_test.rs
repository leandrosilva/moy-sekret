use moy_sekret::create_keypair;
use std::fs;
use std::panic;
use std::path::Path;

// Fixtures
//

const F_KEYS_DIR: &str = "test_keys";
const F_USER: &str = "tester";

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
        Err(reason) => assert!(false , format!("Failed to create test keys' directory: {}", reason)),
    }
}

fn remove_keys_dir() {
    let keys_dir = Path::new(F_KEYS_DIR);
    let _ = fs::remove_dir_all(keys_dir);
}

// Tests
//

#[test]
fn should_create_keypair_for_user_and_save_it_to_a_given_directory() {
    run_test(|| {
        let keys_dir = F_KEYS_DIR.to_string();
        let user = F_USER.to_string();

        match create_keypair(&keys_dir, &user) {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, format!("Should have created but: {}", e)),
        }
    })
}

#[test]
fn should_not_create_keypair_due_to_permission_denied_on_keys_directory() {
    // Have to find out how to test it on Windows but not now
    if cfg!(windows) {
        assert!(true);
        return
    }

    run_test(|| {
        let keys_dir = String::from("/keys");
        let user = F_USER.to_string();

        match create_keypair(&keys_dir, &user) {
            Ok(_) => assert!(false, "Should not create key pair"),
            Err(e) => assert_eq!("Could not save pk file: Failed to create keys' directory: Permission denied (os error 13)", e.to_string()),
        }
    })
}
