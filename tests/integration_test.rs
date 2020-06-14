use moy_sekret::{create_keypair};

#[test]
fn create_keypair_for_user_and_save_to_given_directory() {
    let keys_dir = String::from("keys");
    let user = String::from("leandro");

    match create_keypair(&keys_dir, &user) {
        Ok(_) => assert!(true),
        Err(e) => assert!(false, format!("Should have created but: {}", e)),
    }
}

#[test]
fn cannot_create_keypair_due_to_permission_denied_on_keys_directory() {
    let keys_dir = String::from("/keys");
    let user = String::from("leandro");

    match create_keypair(&keys_dir, &user) {
        Ok(_) => assert!(false, "Should not create key pair"),
        Err(e) => assert_eq!("Could not save pk file: Failed to create keys' directory: Permission denied (os error 13)", e.to_string()),
    }
}
