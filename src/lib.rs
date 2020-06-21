use data_encoding::BASE64;
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
use std::process;

// Custom types
//

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub name: String,
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

pub fn init(storage_dir: &String, profile: &String, should_override: bool) -> Result<(), AnyError> {
    if !should_override {
        if let Ok(()) = profile_exists(storage_dir, profile) {
            return error_without_parent("Initialization failed because profile already exists");
        }
    }

    match create_storage_dir(&storage_dir) {
        Ok(_) => (),
        Err(reason) => return error("Initialization failed while creating storage for files", reason),
    };

    match create_profile(storage_dir, profile) {
        Ok(_) => (),
        Err(reason) => return error("Initialization failed while creating profile", reason),
    };

    match create_keypair(&storage_dir, &profile) {
        Ok(_) => (),
        Err(reason) => return error("Initialization failed while creating key pair", reason),
    }

    Ok(())
}

pub fn encrypt(_profile: &String, _file_path: &String, _should_override: bool) -> Result<(), AnyError> {
    error_without_parent("Not implemented yet")
}

pub fn decrypt(_profile: &String, _file_path: &String, _should_override: bool) -> Result<(), AnyError> {
    error_without_parent("Not implemented yet")
}

// Business functions
//

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

pub fn profile_exists(storage_dir: &String, profile: &String) -> Result<(), AnyError> {
    if !profile_file_exists(storage_dir, profile) {
        return error_without_parent("Profile file does not exist");
    }
    keypair_exists(storage_dir, profile)
}

fn create_profile(storage_dir: &String, profile: &String) -> Result<(), AnyError> {
    let profile_obj = Profile {
        name: profile.to_owned(),
    };

    let profile_file_path = format!("{}/{}.toml", storage_dir, profile);
    match save_profile(&profile_obj, &profile_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save profile file", reason),
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

pub fn keypair_exists(storage_dir: &String, profile: &String) -> Result<(), AnyError> {
    if !key_file_exists(storage_dir, profile, Key::PublicKey) {
        return error_without_parent("Public key file does not exist");
    }
    if !key_file_exists(storage_dir, profile, Key::SecretKey) {
        return error_without_parent("Secret key file does not exist");
    }
    Ok(())
}

fn create_keypair(storage_dir: &String, profile: &String) -> Result<(PublicKey, SecretKey), AnyError> {
    let (pk, sk) = box_::gen_keypair();

    let pk_file_path = format!("{}/{}.pk", storage_dir, profile);
    match save_key(pk.as_ref(), &pk_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save public key file", reason),
    };

    let sk_file_path = format!("{}/{}.sk", storage_dir, profile);
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

fn profile_file_exists(storage_dir: &String, profile: &String) -> bool {
    let file_path = format!("{}/{}.toml", storage_dir, profile);
    let file = Path::new(file_path.as_str());
    file.is_file()
}

fn key_file_exists(storage_dir: &String, profile: &String, key: Key) -> bool {
    let file_path = format!("{}/{}.{}", storage_dir, profile, key);
    let file = Path::new(file_path.as_str());
    file.is_file()
}

// Helper functions
//

pub fn exit_with_error(message: &str, reason: AnyError) {
    eprintln!("{}: {}", message, reason);
    process::exit(666);
}

// Unit tests
//

#[cfg(test)]
mod unit_test;
