use data_encoding::BASE64;
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

pub enum CryptoKey {
    PublicKey,
    SecretKey,
}

impl fmt::Display for CryptoKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CryptoKey::PublicKey => write!(f, "pk"),
            CryptoKey::SecretKey => write!(f, "sk"),
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

pub fn init(keys_dir: &String, profile: &String, should_override: bool) -> Result<(), AnyError> {
    if !should_override {
        if let Ok(()) = profile_exists(keys_dir, profile) {
            return error_without_parent("Initialization failed because profile already exists");
        }
    }
    match create_keypair(&keys_dir, &profile) {
        Ok(_) => Ok(()),
        Err(reason) => error("Initialization failed while creating key pair", reason),
    }
}

pub fn encrypt(_file_path: &String, _should_override: bool) -> Result<(), AnyError> {
    error_without_parent("Not implemented yet")
}

pub fn decrypt(_file_path: &String, _should_override: bool) -> Result<(), AnyError> {
    error_without_parent("Not implemented yet")
}

// Business functions
//

pub fn profile_exists(keys_dir: &String, profile: &String) -> Result<(), AnyError> {
    if !profile_file_exists(keys_dir, profile) {
        return error_without_parent("Profile file does not exist");
    }
    keypair_exists(keys_dir, profile)
}

pub fn keypair_exists(keys_dir: &String, profile: &String) -> Result<(), AnyError> {
    if !key_file_exists(keys_dir, profile, CryptoKey::PublicKey) {
        return error_without_parent("Public key file does not exist");
    }
    if !key_file_exists(keys_dir, profile, CryptoKey::SecretKey) {
        return error_without_parent("Secret key file does not exist");
    }
    Ok(())
}

fn create_keypair(keys_dir: &String, profile: &String) -> Result<(PublicKey, SecretKey), AnyError> {
    match create_keys_dir_if_not_exists(&keys_dir) {
        Ok(_) => (),
        Err(reason) => return error("Could not create keys directory", reason),
    };

    let (pk, sk) = box_::gen_keypair();

    let pk_file_path = format!("{}/{}.pk", keys_dir, profile);
    match save_key(pk.as_ref(), &pk_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save public key file", reason),
    };

    let sk_file_path = format!("{}/{}.sk", keys_dir, profile);
    match save_key(sk.as_ref(), &sk_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save secret key file", reason),
    };

    Ok((pk, sk))
}

fn create_keys_dir_if_not_exists(keys_dir: &String) -> Result<(), AnyError> {
    let path = Path::new(keys_dir);
    if path.is_dir() {
        return Ok(());
    }

    match fs::create_dir_all(path) {
        Ok(_) => Ok(()),
        Err(reason) => error("Failed to create keys' directory", reason),
    }
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

fn profile_file_exists(_keys_dir: &String, _profile: &String) -> bool {
    true
}

fn key_file_exists(keys_dir: &String, profile: &String, key: CryptoKey) -> bool {
    let file_path = format!("{}/{}.{}", keys_dir, profile, key);
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
