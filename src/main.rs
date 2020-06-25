use clap::{App, Arg};
use console::Style;
use dialoguer::Confirm;
use moy_sekret::{decrypt, encrypt, exit_normal, exit_with_error, init, AnyError};

// Macros
//

macro_rules! confirm_override {
    ($warning_override:expr, $warning_unrecoverable:expr) => {
        let red_alert = Style::new().red();
        println!(
            concat!($warning_override, "\n", $warning_unrecoverable),
            OVERRIDE = red_alert.apply_to("override"),
            UNRECOVERABLE = red_alert.apply_to("unrecoverable")
        );
        let confirm = Confirm::new()
            .with_prompt("Are you sure about that?")
            .interact();
        if let Ok(false) = confirm {
            exit_normal("Okay. Safe move.");
        }
    };
}

// Main
//

fn main() {
    let profile_arg = Arg::with_name("profile")
        .about("name of the profile")
        .short('p')
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
                        .about("target directory where to store keys and encrypted files")
                        .short('d')
                        .long("dir")
                        .takes_value(true)
                        .value_name("DIR")
                        .required(true),
                )
                .arg(
                    Arg::with_name("override")
                        .about("Should it override existing profile and keys or not")
                        .short('o')
                        .long("override"),
                ),
        )
        .subcommand(
            App::new("encrypt")
                .about("Encrypts a source file, saves it to the target repository directory and keeps the original one.")
                .arg(
                    &profile_arg,
                )
                .arg(
                    Arg::with_name("file")
                        .about("path to the source file to be encrypted")
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
                .about("Decrypts a source file, saves it plain to a target directory and keeps the encrypted one.")
                .arg(
                    &profile_arg,
                )
                .arg(
                    Arg::with_name("file")
                        .about("path to the source file to be decrypted")
                        .short('f')
                        .long("file")
                        .takes_value(true)
                        .value_name("FILE")
                        .required(true),
                )
                .arg(
                    Arg::with_name("dest")
                        .about("target directory to where save the decrypted file")
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
        ("init", Some(sub_matches)) => {
            let should_override = sub_matches.is_present("override");
            if should_override {
                confirm_override!(
                    "This operation will {OVERRIDE} any key you have got with this profile.",
                    "This is {UNRECOVERABLE} and you may lose access to any file you have encrypted with those keys."
                );
            }

            let profile = sub_matches.value_of("profile").unwrap().to_owned();
            let storage_dir = sub_matches.value_of("dir").unwrap().to_owned();

            match init(&profile, &storage_dir, should_override) {
                Ok(()) => println!(
                    "Key pair created with success at {} directory",
                    &storage_dir
                ),
                Err(reason) => generic_exit_with_error(reason),
            }
        }
        ("encrypt", Some(sub_matches)) => {
            let should_override = sub_matches.is_present("override");
            if should_override {
                confirm_override!(
                    "This operation will {OVERRIDE} the existing encrypted file.",
                    "This is {UNRECOVERABLE}, please be sure what you are about to do."
                );
            }

            let profile = sub_matches.value_of("profile").unwrap().to_owned();
            let file_path = sub_matches.value_of("file").unwrap().to_owned();

            match encrypt(&profile, &file_path, should_override) {
                Ok(()) => println!("Encryption succesfully done"),
                Err(reason) => generic_exit_with_error(reason),
            }
        }
        ("decrypt", Some(sub_matches)) => {
            let should_override = sub_matches.is_present("override");
            if should_override {
                confirm_override!(
                    "This operation will {OVERRIDE} the existing plain file.",
                    "This is {UNRECOVERABLE}, please be sure what you are about to do."
                );
            }

            let profile = sub_matches.value_of("profile").unwrap().to_owned();
            let file_path = sub_matches.value_of("file").unwrap().to_owned();
            let dest_dir = sub_matches.value_of("dest").unwrap().to_owned();

            match decrypt(&profile, &file_path, &dest_dir, should_override) {
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
