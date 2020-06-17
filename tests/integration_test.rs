use moy_sekret::{init, profile_exists};
use std::fs;
use std::panic;
use std::path::Path;

// Fixtures
//

const F_KEYS_DIR: &str = "./test_keys";
const F_PROFILE: &str = "tester";
const F_OVERRIDE_PROFILE: bool = false;

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

        match init(&keys_dir, &profile, F_OVERRIDE_PROFILE) {
            Ok(_) => match profile_exists(&keys_dir, &profile) {
                Ok(_) => assert!(true),
                Err(reason) => assert!(
                    false,
                    format!(
                        "Should exist profile {} in {} directory but: {}",
                        &profile, &keys_dir, reason
                    )
                ),
            },
            Err(e) => assert!(false, format!("Should have initiated but: {}", e)),
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

        match init(&keys_dir, &profile, F_OVERRIDE_PROFILE) {
            Ok(_) => assert!(false, "Should have not initiated"),
            Err(reason) => {
                assert_eq!(
                    "Initialization failed while creating key pair: Could not create keys directory: Failed to create keys' directory: Permission denied (os error 13)",
                    reason.to_string()
                );
                match profile_exists(&keys_dir, &profile) {
                    Ok(_) => assert!(
                        false,
                        format!(
                            "Should not exist profile {} in {} directory",
                            &profile, &keys_dir
                        )
                    ),
                    Err(_) => assert!(true),
                };
            }
        }
    })
}

#[test]
fn should_init_when_profile_exists_and_override_flag_is_present() {
    run_test(|| {
        let keys_dir = F_KEYS_DIR.to_string();
        let profile = F_PROFILE.to_string();

        match init(&keys_dir, &profile, F_OVERRIDE_PROFILE) {
            Ok(_) => {
                let flag_override_profile = true;
                match init(&keys_dir, &profile, flag_override_profile) {
                    Ok(_) => assert!(true),
                    Err(reason) => assert_eq!("Should have initiated and overridden existent profile but:", reason.to_string()),
                }
            }
            Err(e) => assert!(false, format!("Should have initiated profile but: {}", e)),
        }
    })
}

#[test]
fn should_not_init_when_profile_exists_and_override_flag_is_not_present() {
    run_test(|| {
        let keys_dir = F_KEYS_DIR.to_string();
        let profile = F_PROFILE.to_string();

        match init(&keys_dir, &profile, F_OVERRIDE_PROFILE) {
            Ok(_) => {
                let flag_override_profile = false;
                match init(&keys_dir, &profile, flag_override_profile) {
                    Ok(_) => assert!(
                        false,
                        "Should not initialize an existent profile when override flag is not present"
                    ),
                    Err(reason) => assert_eq!("Initialization failed because profile already exists", reason.to_string()),
                }
            }
            Err(e) => assert!(false, format!("Should have initiated profile but: {}", e)),
        }
    })
}
