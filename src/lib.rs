use data_encoding::BASE64;
use dirs;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_;
use sodiumoxide::crypto::box_::curve25519xsalsa20poly1305::Nonce;
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

#[derive(Serialize, Deserialize, Debug)]
struct Cipher {
    nonce: Nonce,
    data: Vec<u8>,
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

pub fn init(
    profile_name: &String,
    storage_dir: &String,
    should_override: bool,
) -> Result<(), AnyError> {
    if !should_override {
        if profile_exists(&profile_name) {
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

    let profile = match create_profile(&profile_name, &abs_storage_dir) {
        Ok(obj) => obj,
        Err(reason) => return error("Initialization failed while creating profile", reason),
    };

    match create_keypair(&profile) {
        Ok(_) => (),
        Err(reason) => return error("Initialization failed while creating key pair", reason),
    }

    Ok(())
}

pub fn encrypt(
    profile_name: &String,
    file_path: &String,
    should_override: bool,
) -> Result<(), AnyError> {
    if file_path.ends_with(".cz") {
        return error_without_parent(
            "Encryption failed because source file was already encrypted by this program (.cz)",
        );
    }

    if !file_exists(&file_path) {
        return error_without_parent("Encryption failed because source file does not exists");
    }

    let profile = match read_profile(&profile_name) {
        Ok(obj) => obj,
        Err(reason) => return error("Encryption failed while reading user profile", reason),
    };

    let encrypted_file_path = get_encrypted_file_name(&profile, &file_path);
    if !should_override {
        if file_exists(&encrypted_file_path) {
            return error_without_parent("Encryption failed because target file already exists");
        }
    }

    match encrypt_file(&profile, &file_path) {
        Ok(_) => (),
        Err(reason) => return error("Encryption failed while doing actual encryption", reason),
    };

    Ok(())
}

pub fn decrypt(
    profile_name: &String,
    file_path: &String,
    dest_dir: &String,
    should_override: bool,
) -> Result<(), AnyError> {
    if !file_path.ends_with(".cz") {
        return error_without_parent(
            "Decryption failed because source file was not made by this program (.cz)",
        );
    }

    if !file_exists(&file_path) {
        return error_without_parent("Decryption failed because source file does not exists");
    }

    let decrypted_file_path = get_decrypted_file_name(&file_path, &dest_dir);
    if !should_override {
        if file_exists(&decrypted_file_path) {
            return error_without_parent("Decryption failed because target file already exists");
        }
    }

    let profile = match read_profile(&profile_name) {
        Ok(obj) => obj,
        Err(reason) => return error("Decryption failed while reading user profile", reason),
    };

    match decrypt_file(&profile, &file_path, &dest_dir) {
        Ok(_) => (),
        Err(reason) => return error("Decryption failed while doing actual decryption", reason),
    };

    Ok(())
}

// Business functions
//

// -- Profile

pub fn profile_exists(profile_name: &String) -> bool {
    profile_file_exists(profile_name)
}

fn read_profile(profile_name: &String) -> Result<Profile, AnyError> {
    let file_name = get_profile_file_name(&profile_name);
    match fs::read_to_string(file_name) {
        Ok(content) => {
            let result = toml::from_str(content.as_str());
            match result {
                Ok(profile) => Ok(profile),
                Err(reason) => error("Could not parse profile file", reason),
            }
        }
        Err(reason) => error("Could not read profile", reason),
    }
}

fn create_profile(profile_name: &String, storage_dir: &String) -> Result<Profile, AnyError> {
    let profile = Profile {
        name: profile_name.to_owned(),
        storage: storage_dir.to_owned(),
    };

    let profile_file_path = get_profile_file_name(&profile_name);
    match save_profile(&profile, &profile_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save profile", reason),
    };

    Ok(profile)
}

fn save_profile(profile: &Profile, output_file_path: &String) -> Result<(), AnyError> {
    let profile_file_path = Path::new(output_file_path.as_str());

    let mut key_file = match File::create(profile_file_path) {
        Ok(file) => file,
        Err(reason) => return error("Could not create profile file", reason),
    };

    let profile_ser = toml::to_string(&profile).unwrap();

    match key_file.write_all(profile_ser.as_bytes()) {
        Ok(_) => (),
        Err(reason) => return error("Could not write profile file", reason),
    };

    Ok(())
}

fn get_profile_file_name(profile_name: &String) -> String {
    match dirs::home_dir() {
        Some(path) => format!("{}/.moy-sekret.{}.toml", path.display(), profile_name),
        None => format!(".moy-sekret.{}.toml", profile_name),
    }
}

fn profile_file_exists(profile_name: &String) -> bool {
    let file_path = get_profile_file_name(&profile_name);
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

pub fn keypair_exists(profile: &Profile) -> bool {
    if !key_file_exists(&profile, Key::PublicKey) {
        return false;
    }
    if !key_file_exists(&profile, Key::SecretKey) {
        return false;
    }
    true
}

fn read_keypair(profile: &Profile) -> Result<Keypar, AnyError> {
    let pk_file_path = get_key_file_name(&profile, Key::PublicKey);
    let pk = match read_key(&pk_file_path) {
        Ok(raw) => match PublicKey::from_slice(raw.as_ref()) {
            Some(pk_obj) => pk_obj,
            None => return error_without_parent("Could not decode public key"),
        },
        Err(reason) => return error("Could not read public key", reason),
    };

    let sk_file_path = get_key_file_name(&profile, Key::SecretKey);
    let sk = match read_key(&sk_file_path) {
        Ok(raw) => match SecretKey::from_slice(raw.as_ref()) {
            Some(sk_obj) => sk_obj,
            None => return error_without_parent("Could not decode secret key"),
        },
        Err(reason) => return error("Could not read public key", reason),
    };
    Ok((pk, sk))
}

fn read_key(input_file_path: &String) -> Result<Vec<u8>, AnyError> {
    let key_file_path = Path::new(input_file_path.as_str());
    match fs::read_to_string(key_file_path) {
        Ok(raw_base64) => match BASE64.decode(raw_base64.as_bytes()) {
            Ok(raw_vec) => Ok(raw_vec),
            Err(reason) => error("Could not decode key file", reason),
        },
        Err(reason) => error("Could not read key file", reason),
    }
}

fn create_keypair(profile: &Profile) -> Result<Keypar, AnyError> {
    let (pk, sk) = box_::gen_keypair();

    let pk_file_path = get_key_file_name(&profile, Key::PublicKey);
    match save_key(pk.as_ref(), &pk_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save public key file", reason),
    };

    let sk_file_path = get_key_file_name(&profile, Key::SecretKey);
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

fn get_key_file_name(profile: &Profile, key: Key) -> String {
    format!("{}/{}.{}", profile.storage, profile.name, key)
}

fn key_file_exists(profile: &Profile, key: Key) -> bool {
    let file_path = get_key_file_name(&profile, key);
    let file = Path::new(file_path.as_str());
    file.is_file()
}

// -- Encryption

fn encrypt_file(profile: &Profile, file_path: &String) -> Result<(), AnyError> {
    let (pk, sk) = match read_keypair(&profile) {
        Ok(keypair) => keypair,
        Err(reason) => return error("Could not encrypt file", reason),
    };

    let plain_content = match fs::read(file_path) {
        Ok(raw_vec) => raw_vec,
        Err(reason) => return error("Could not read file to encrypt", reason),
    };

    let nonce = box_::gen_nonce();
    let cipher_data = box_::seal(plain_content.as_ref(), &nonce, &pk, &sk);

    let cipher_file_path = get_encrypted_file_name(&profile, &file_path);
    let cipher = Cipher {
        nonce: nonce,
        data: cipher_data,
    };

    match save_encrypted_file(&cipher, &cipher_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save encrypted file", reason),
    };

    Ok(())
}

fn save_encrypted_file(cipher: &Cipher, output_file_path: &String) -> Result<(), AnyError> {
    let cipher_file_path = Path::new(output_file_path);

    let mut cipher_file = match File::create(cipher_file_path) {
        Ok(file) => file,
        Err(reason) => return error("Could not create encrypted file", reason),
    };

    let cipher_data = match bincode::serialize(cipher) {
        Ok(data) => data,
        Err(reason) => return error("Could not serialize encrypted data", reason),
    };

    match cipher_file.write_all(&cipher_data) {
        Ok(_) => (),
        Err(reason) => return error("Could not write to encrypted file", reason),
    };

    Ok(())
}

fn get_encrypted_file_name(profile: &Profile, file_name: &String) -> String {
    let path = Path::new(file_name);
    let name = path.file_name().unwrap();
    format!("{}/{}.cz", profile.storage, name.to_str().unwrap())
}

// -- Decryption

// -- Encryption

fn decrypt_file(profile: &Profile, file_path: &String, dest_dir: &String) -> Result<(), AnyError> {
    let (pk, sk) = match read_keypair(&profile) {
        Ok(keypair) => keypair,
        Err(reason) => return error("Could not encrypt file", reason),
    };

    let cipher_content = match fs::read(file_path) {
        Ok(raw_vec) => raw_vec,
        Err(reason) => return error("Could not read file to decrypt", reason),
    };

    let cipher: Cipher = match bincode::deserialize(&cipher_content) {
        Ok(data) => data,
        Err(reason) => return error("Could not deserialize encrypted data", reason),
    };

    let plain_data = match box_::open(cipher.data.as_ref(), &cipher.nonce, &pk, &sk) {
        Ok(data) => data,
        Err(_) => return error_without_parent("Could not decrypt file"),
    };

    let plain_file_path = get_decrypted_file_name(&file_path, &dest_dir);
    match save_decrypted_file(&plain_data, &plain_file_path) {
        Ok(_) => (),
        Err(reason) => return error("Could not save decrypted file", reason),
    };

    Ok(())
}

fn save_decrypted_file(plain_data: &[u8], output_file_path: &String) -> Result<(), AnyError> {
    let plain_file_path = Path::new(output_file_path);

    create_dir_if_not_exists(&format!("{}", plain_file_path.parent().unwrap().display()))?;

    let mut plain_file = match File::create(plain_file_path) {
        Ok(file) => file,
        Err(reason) => return error("Could not create plain file", reason),
    };

    match plain_file.write_all(&plain_data) {
        Ok(_) => (),
        Err(reason) => return error("Could not write to plain file", reason),
    };

    Ok(())
}

fn get_decrypted_file_name(file_name: &String, dest_dir: &String) -> String {
    let path = Path::new(file_name);
    let name = path.file_stem().unwrap();
    format!("{}/{}", dest_dir, name.to_str().unwrap())
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

fn create_dir_if_not_exists(given_dir: &String) -> Result<(), AnyError> {
    let path = Path::new(given_dir);
    if !path.exists() {
        match fs::create_dir_all(path) {
            Ok(_) => return Ok(()),
            Err(reason) => return error("Could not create directory", reason),
        }
    }
    Ok(())
}

// Unit tests
//

#[cfg(test)]
mod unit_test;
