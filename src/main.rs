use moy_sekret::{create_keypair, exit_with_error};

// Main
//

fn main() {
    let keys_dir = String::from("keys");
    let user = String::from("leandro");

    let (_pk, _sk) = match create_keypair(&keys_dir, &user) {
        Ok(keypair) => keypair,
        Err(reason) => {
            exit_with_error("Something went really bad here", reason);
            return ();
        }
    };

    println!("Key pair created with success at {} directory", keys_dir);
}
