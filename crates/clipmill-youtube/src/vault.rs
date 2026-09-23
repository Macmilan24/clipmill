//! Secrets never travel through argv, renderer state, project archives or logs.
use crate::Error;
#[cfg(target_os = "macos")]
const SERVICE: &str = "dev.clipmill.ClipMill.youtube";

pub const fn supported() -> bool {
    cfg!(target_os = "macos")
}

#[cfg(target_os = "macos")]
pub fn put(key: &str, value: &[u8]) -> Result<(), Error> {
    security_framework::passwords::set_generic_password(SERVICE, key, value)
        .map_err(|_| Error::CredentialStore)
}
#[cfg(target_os = "macos")]
pub fn get(key: &str) -> Result<Vec<u8>, Error> {
    optional(key)?.ok_or(Error::MissingCredential)
}
#[cfg(target_os = "macos")]
pub fn optional(key: &str) -> Result<Option<Vec<u8>>, Error> {
    match security_framework::passwords::get_generic_password(SERVICE, key) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.code() == -25300 => Ok(None), // errSecItemNotFound
        Err(_) => Err(Error::CredentialStore),
    }
}
#[cfg(target_os = "macos")]
pub fn delete(key: &str) -> Result<(), Error> {
    match security_framework::passwords::delete_generic_password(SERVICE, key) {
        Ok(()) => Ok(()),
        Err(error) if error.code() == -25300 => Ok(()),
        Err(_) => Err(Error::CredentialStore),
    }
}
#[cfg(not(target_os = "macos"))]
pub fn put(_key: &str, _value: &[u8]) -> Result<(), Error> {
    Err(Error::UnsupportedPlatform)
}
#[cfg(not(target_os = "macos"))]
pub fn get(_key: &str) -> Result<Vec<u8>, Error> {
    Err(Error::UnsupportedPlatform)
}
#[cfg(not(target_os = "macos"))]
pub fn optional(_key: &str) -> Result<Option<Vec<u8>>, Error> {
    Err(Error::UnsupportedPlatform)
}
#[cfg(not(target_os = "macos"))]
pub fn delete(_key: &str) -> Result<(), Error> {
    Err(Error::UnsupportedPlatform)
}
