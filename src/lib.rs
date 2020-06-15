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

// Custom error types
//

type DynError = Box<dyn Error>;
type OptError = Option<DynError>;

#[derive(Debug)]
pub struct AnyError {
    pub details: String,
    pub reason: OptError,
}

impl AnyError {
    fn new(details: &str, reason: OptError) -> AnyError {
        AnyError {
            details: details.to_string(),
            reason: reason,
        }
    }

    #[allow(dead_code)]
    fn without_cause(details: &str) -> AnyError {
        AnyError::new(details, None)
    }
}

impl Error for AnyError {
}

impl fmt::Display for AnyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.reason {
            Some(c) => write!(f, "{}: {}", self.details, c),
            None => write!(f, "{}", self.details),
        }
    }
}

pub fn error<T, U: 'static + Error>(message: &str, reason: U) -> Result<T, AnyError> {
    return Err(AnyError::new(&message, Some(Box::new(reason))))
}

// Business functions
//

pub fn create_keypair(keys_dir: &String, user: &String) -> Result<(PublicKey, SecretKey), AnyError> {
    let (pk, sk) = box_::gen_keypair();

    let pk_file_path = format!("{}/{}.pk", keys_dir, user);
    match save_key(pk.as_ref(), &pk_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save pk file", reason),
    };

    let sk_file_path = format!("{}/{}.sk", keys_dir, user);
    match save_key(sk.as_ref(), &sk_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save secret file: {}", reason),
    };

    return Ok((pk, sk));
}

fn save_key(key: &[u8], output_file_path: &String) -> Result<(), AnyError> {
    let key_file_path = Path::new(output_file_path.as_str());

    match fs::create_dir_all(key_file_path.parent().unwrap()) {
        Ok(_) => (),
        Err(reason) => return error("Failed to create keys' directory", reason),
    };

    let mut key_file = match File::create(key_file_path) {
        Ok(file) => file,
        Err(reason) => return error("Could not create key file", reason),
    };

    let key_file_base64 = BASE64.encode(key);

    match key_file.write_all(key_file_base64.as_bytes()) {
        Ok(_) => (),
        Err(reason) => return error("Could not create key file", reason),
    };

    return Ok(());
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