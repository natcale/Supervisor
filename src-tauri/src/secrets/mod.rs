/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use std::path::Path;

const SERVICE: &str = "com.supervisor.app";
const ACCOUNT: &str = "nexus-api-key";

pub fn get_nexus_api_key() -> AppResult<Option<String>> {
    match keyring::Entry::new(SERVICE, ACCOUNT)?.get_password() {
        Ok(value) if value.trim().is_empty() => Ok(None),
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Keyring(e)),
    }
}

pub fn set_nexus_api_key(key: &str) -> AppResult<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        return delete_nexus_api_key();
    }
    keyring::Entry::new(SERVICE, ACCOUNT)?
        .set_password(trimmed)
        .map_err(AppError::Keyring)
}

pub fn delete_nexus_api_key() -> AppResult<()> {
    match keyring::Entry::new(SERVICE, ACCOUNT)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Keyring(e)),
    }
}

pub fn has_nexus_api_key() -> bool {
    get_nexus_api_key()
        .ok()
        .flatten()
        .is_some_and(|k| !k.trim().is_empty())
}

/// Move legacy plain-text API keys from settings.json into the OS credential store.
pub fn migrate_nexus_api_key_from_settings(app_data: &Path) -> AppResult<()> {
    let path = app_data.join("settings.json");
    if !path.is_file() {
        return Ok(());
    }
    let raw = std::fs::read_to_string(&path).map_err(AppError::Io)?;
    let mut value: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| AppError::user(format!("Corrupt settings: {e}")))?;
    let Some(key) = value
        .get("nexusApiKey")
        .and_then(|v| v.as_str())
        .filter(|k| !k.trim().is_empty())
        .map(str::to_string)
    else {
        return Ok(());
    };

    if !has_nexus_api_key() {
        set_nexus_api_key(&key)?;
    }
    if let Some(obj) = value.as_object_mut() {
        obj.remove("nexusApiKey");
    }
    std::fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).map_err(AppError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_key_is_not_present() {
        assert!(!has_nexus_api_key() || get_nexus_api_key().ok().flatten().is_some());
    }
}
