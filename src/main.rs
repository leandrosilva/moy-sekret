use clap::{App, Arg};
use moy_sekret::{decrypt, encrypt, exit_with_error, init, AnyError};

// Main
//

fn main() {
    let profile_arg = Arg::with_name("profile")
        .about("name of the profile")
        .short('u')
        .long("profile")
        .takes_value(true)
        .value_name("PROFILE")
        .required(true);
    let mut app = App::new("Moy Sekret")
        .version("1.0")
        .author("Leandro Silva <leandrodoze@gmail.com>")
        .about("You know, that is kind of... secret.")
        .subcommand(
            App::new("init")
                .about("Initializes the app for a give profile.")
                .arg(
                    &profile_arg,
                )
                .arg(
                    Arg::with_name("dir")
                        .about("directory where to place keys and encrypted files")
                        .short('d')
                        .long("dir")
                        .takes_value(true)
                        .value_name("DIR")
                        .required(true),
                )
                .arg(
                    Arg::with_name("override")
                        .about("Should it override existing profile or not")
                        .short('o')
                        .long("override"),
                ),
        )
        .subcommand(
            App::new("encrypt")
                .about("Encrypts a file, saves it to the repository directory and deletes the original one.")
                .arg(
                    &profile_arg,
                )
                .arg(
                    Arg::with_name("file")
                        .about("path to the file to be encrypted")
                        .short('f')
                        .long("file")
                        .takes_value(true)
                        .value_name("FILE")
                        .required(true),
                )
                .arg(
                    Arg::with_name("override")
                        .about("Should it override existing encrypted file or not")
                        .short('o')
                        .long("override"),
                ),
        )
        .subcommand(
            App::new("decrypt")
                .about("Decrypts a file, saves it plain to given directory but keeps the encrypted one.")
                .arg(
                    &profile_arg,
                )
                .arg(
                    Arg::with_name("file")
                        .about("name of the encrypted file")
                        .short('f')
                        .long("file")
                        .takes_value(true)
                        .value_name("FILE")
                        .required(true),
                )
                .arg(
                    Arg::with_name("dest")
                        .about("directory to where save the decrypted file")
                        .short('d')
                        .long("dest")
                        .takes_value(true)
                        .value_name("DEST")
                        .default_value(".")
                )
                .arg(
                    Arg::with_name("override")
                        .about("Should it override existing plain file or not")
                        .short('o')
                        .long("override"),
                ),
        );

    let matches = app.get_matches_mut();
    match matches.subcommand() {
        ("init", Some(init_matches)) => {
            let keys_dir = init_matches.value_of("dir").unwrap().to_owned();
            let profile = init_matches.value_of("profile").unwrap().to_owned();
            let should_override = init_matches.is_present("override");

            match init(&keys_dir, &profile, should_override) {
                Ok(()) => println!("Key pair created with success at {} directory", &keys_dir),
                Err(reason) => generic_exit_with_error(reason),
            }
        }
        ("encrypt", Some(encrypt_matches)) => {
            let file_path = encrypt_matches.value_of("file").unwrap().to_owned();
            let should_override = encrypt_matches.is_present("override");

            match encrypt(&file_path, should_override) {
                Ok(()) => println!("Encryption succesfully done"),
                Err(reason) => generic_exit_with_error(reason),
            }
        }
        ("decrypt", Some(decrypt_matches)) => {
            let file_path = decrypt_matches.value_of("file").unwrap().to_owned();
            let should_override = decrypt_matches.is_present("override");

            match decrypt(&file_path, should_override) {
                Ok(()) => println!("Decryption succesfully done"),
                Err(reason) => generic_exit_with_error(reason),
            }
        }
        ("", None) => app.print_help().unwrap(),
        _ => unreachable!(),
    }
}

fn generic_exit_with_error(reason: AnyError) {
    // Should give it a real better implementation any time soon
    exit_with_error("Something went really bad here", reason);
}
