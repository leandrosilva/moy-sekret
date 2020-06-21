use data_encoding::BASE64;
use dirs;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305::PublicKey;
use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305::SecretKey;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::process;

// Custom types
//

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub name: String,
    pub storage: String,
}

#[derive(Debug)]
pub enum Key {
    PublicKey,
    SecretKey,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Key::PublicKey => write!(f, "pk"),
            Key::SecretKey => write!(f, "sk"),
        }
    }
}

pub type Keypar = (PublicKey, SecretKey);

// Custom error types
//

type DynError = Box<dyn Error>;
type OptError = Option<DynError>;

#[derive(Debug)]
pub struct AnyError {
    pub details: String,
    pub parent: OptError,
}

impl AnyError {
    fn new(details: &str, reason: OptError) -> AnyError {
        AnyError {
            details: details.to_string(),
            parent: reason,
        }
    }

    fn without_parent(details: &str) -> AnyError {
        AnyError::new(details, None)
    }
}

impl Error for AnyError {}

impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.parent {
            Some(c) => write!(f, "{}: {}", self.details, c),
            None => write!(f, "{}", self.details),
        }
    }
}

pub fn error<T, U: 'static + Error>(message: &str, reason: U) -> Result<T, AnyError> {
    Err(AnyError::new(&message, Some(Box::new(reason))))
}

pub fn error_without_parent<T>(message: &str) -> Result<T, AnyError> {
    Err(AnyError::without_parent(&message))
}

// Entrypoint functions
//

pub fn init(profile: &String, storage_dir: &String, should_override: bool) -> Result<(), AnyError> {
    if !should_override {
        if profile_exists(&profile) {
            return error_without_parent("Initialization failed because profile already exists");
        }
    }

    match create_storage_dir(&storage_dir) {
        Ok(_) => (),
        Err(reason) => {
            return error(
                "Initialization failed while creating storage for files",
                reason,
            )
        }
    };

    let abs_storage_dir = expand_storage_dir(&storage_dir)?;

    match create_profile(&profile, &abs_storage_dir) {
        Ok(_) => (),
        Err(reason) => return error("Initialization failed while creating profile", reason),
    };

    match create_keypair(&profile, &abs_storage_dir) {
        Ok(_) => (),
        Err(reason) => return error("Initialization failed while creating key pair", reason),
    }

    Ok(())
}

pub fn encrypt(
    profile: &String,
    file_path: &String,
    should_override: bool,
) -> Result<(), AnyError> {
    let profile_obj = match read_profile(&profile) {
        Ok(obj) => obj,
        Err(reason) => return error("Encryption failed while reading user profile", reason),
    };

    let encrypted_file_path = get_encrypted_file_name(&profile_obj.storage, &file_path);
    if !should_override {
        if file_exists(&encrypted_file_path) {
            return error_without_parent("Encryption failed because cipher file already exists");
        }
    }

    match encrypt_file(&profile_obj, &file_path, &encrypted_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Encryption failed while doing actual encryption", reason),
    };

    Ok(())
}

pub fn decrypt(
    _profile: &String,
    _file_path: &String,
    _should_override: bool,
) -> Result<(), AnyError> {
    error_without_parent("Not implemented yet")
}

// Business functions
//

// -- Profile

pub fn profile_exists(profile: &String) -> bool {
    profile_file_exists(profile)
}

fn read_profile(profile: &String) -> Result<Profile, AnyError> {
    let file_name = get_profile_file_name(profile);
    match fs::read_to_string(file_name) {
        Ok(content) => {
            let result = toml::from_str(content.as_str());
            match result {
                Ok(profile_obj) => Ok(profile_obj),
                Err(reason) => error("Could not parse profile file", reason),
            }
        }
        Err(reason) => error("Could not read profile", reason),
    }
}

fn create_profile(profile: &String, storage_dir: &String) -> Result<(), AnyError> {
    let profile_obj = Profile {
        name: profile.to_owned(),
        storage: storage_dir.to_owned(),
    };

    let profile_file_path = get_profile_file_name(profile);
    match save_profile(&profile_obj, &profile_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save profile", reason),
    };

    Ok(())
}

fn save_profile(profile_obj: &Profile, output_file_path: &String) -> Result<(), AnyError> {
    let profile_file_path = Path::new(output_file_path.as_str());

    let mut key_file = match File::create(profile_file_path) {
        Ok(file) => file,
        Err(reason) => return error("Could not create profile file", reason),
    };

    let profile_ser = toml::to_string(&profile_obj).unwrap();

    match key_file.write_all(profile_ser.as_bytes()) {
        Ok(_) => (),
        Err(reason) => return error("Could not write profile file", reason),
    };

    Ok(())
}

fn get_profile_file_name(profile: &String) -> String {
    match dirs::home_dir() {
        Some(path) => format!("{}/.moy-sekret.{}.toml", path.display(), profile),
        None => format!(".moy-sekret.{}.toml", profile),
    }
}

fn profile_file_exists(profile: &String) -> bool {
    let file_path = get_profile_file_name(profile);
    let file = Path::new(file_path.as_str());
    file.is_file()
}

// -- Storage

fn storage_dir_exists(storage_dir: &String) -> bool {
    let path = Path::new(storage_dir);
    path.is_dir()
}

fn create_storage_dir(storage_dir: &String) -> Result<(), AnyError> {
    if storage_dir_exists(storage_dir) {
        return Ok(());
    }

    let path = Path::new(storage_dir);
    match fs::create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(reason) => error("Could not create storage directory", reason),
    }
}

fn expand_storage_dir(storage_dir: &String) -> Result<String, AnyError> {
    let path_buf = PathBuf::from(storage_dir);
    match path_buf.canonicalize() {
        Ok(abs_path) => {
            let path = format!("{}", abs_path.display());
            Ok(path)
        }
        Err(reason) => error("Could not expand storage directory", reason),
    }
}

// -- Key pair

pub fn keypair_exists(profile: &String, storage_dir: &String) -> bool {
    if !key_file_exists(profile, storage_dir, Key::PublicKey) {
        return false;
    }
    if !key_file_exists(profile, storage_dir, Key::SecretKey) {
        return false;
    }
    true
}

fn read_keypair(profile: &String, storage_dir: &String) -> Result<Keypar, AnyError> {
    error_without_parent("Not implemented yet")
}

fn create_keypair(
    profile: &String,
    storage_dir: &String,
) -> Result<Keypar, AnyError> {
    let (pk, sk) = box_::gen_keypair();

    let pk_file_path = get_key_file_name(profile, storage_dir, Key::PublicKey);
    match save_key(pk.as_ref(), &pk_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save public key file", reason),
    };

    let sk_file_path = get_key_file_name(profile, storage_dir, Key::SecretKey);
    match save_key(sk.as_ref(), &sk_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save secret key file", reason),
    };

    Ok((pk, sk))
}

fn save_key(key: &[u8], output_file_path: &String) -> Result<(), AnyError> {
    let key_file_path = Path::new(output_file_path.as_str());

    let mut key_file = match File::create(key_file_path) {
        Ok(file) => file,
        Err(reason) => return error("Could not create key file", reason),
    };

    let key_file_base64 = BASE64.encode(key);

    match key_file.write_all(key_file_base64.as_bytes()) {
        Ok(_) => (),
        Err(reason) => return error("Could not write key file", reason),
    };

    Ok(())
}

fn get_key_file_name(profile: &String, storage_dir: &String, key: Key) -> String {
    format!("{}/{}.{}", storage_dir, profile, key)
}

fn key_file_exists(profile: &String, storage_dir: &String, key: Key) -> bool {
    let file_path = get_key_file_name(profile, storage_dir, key);
    let file = Path::new(file_path.as_str());
    file.is_file()
}

// -- Encryption

fn encrypt_file(
    profile_obj: &Profile,
    file_path: &String,
    encrypted_file_path: &String,
) -> Result<(), AnyError> {
    error_without_parent("Not implemented yet")
}

fn get_encrypted_file_name(storage_dir: &String, file_name: &String) -> String {
    format!("{}/{}.cipher", storage_dir, file_name)
}

// Helper functions
//

// -- Process

pub fn exit_normal(message: &str) {
    println!("{}", message);
    process::exit(0);
}

pub fn exit_with_error(message: &str, reason: AnyError) {
    eprintln!("{}: {}", message, reason);
    process::exit(666);
}

// -- File

fn file_exists(file_path: &String) -> bool {
    let path = Path::new(file_path.as_str());
    path.is_file()
}

// Unit tests
//

#[cfg(test)]
mod unit_test;
