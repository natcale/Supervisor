/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NexusModDetails {
    pub mod_id: u64,
    pub name: String,
    pub summary: Option<String>,
    pub picture_url: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub domain: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModUpdateCheck {
    pub mod_id: String,
    pub mod_name: String,
    pub current_version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub nexus_mod_id: u64,
    pub domain: String,
}

#[derive(Debug, Deserialize)]
struct ApiModResponse {
    name: String,
    summary: Option<String>,
    picture_url: Option<String>,
    author: Option<String>,
    version: Option<String>,
    category_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiFileResponse {
    version: Option<String>,
}

pub async fn fetch_mod_details(
    domain: &str,
    mod_id: u64,
    api_key: &str,
) -> AppResult<NexusModDetails> {
    let url = format!("https://api.nexusmods.com/v1/games/{domain}/mods/{mod_id}.json");
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("apikey", api_key)
        .header("Application-Name", "Supervisor")
        .header("Application-Version", env!("CARGO_PKG_VERSION"))
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::user(
            "Invalid Nexus API key. Add a valid key in Settings (from nexusmods.com/users/myaccount?tab=api).",
        ));
    }
    if !response.status().is_success() {
        return Err(AppError::user(format!(
            "Could not fetch mod info from Nexus ({})",
            response.status()
        )));
    }

    let body: ApiModResponse = response.json().await?;
    Ok(NexusModDetails {
        mod_id,
        name: body.name,
        summary: body.summary,
        picture_url: body.picture_url,
        author: body.author,
        version: body.version,
        domain: domain.to_string(),
        category: body.category_name,
    })
}

#[derive(Debug, Clone, Deserialize)]
struct ApiDownloadLink {
    #[serde(default, alias = "URI")]
    uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ApiDownloadLinkResponse {
    #[serde(default)]
    download_link: Option<ApiDownloadLink>,
    #[serde(default)]
    download_links: Vec<ApiDownloadLink>,
}

pub fn build_download_link_url(
    domain: &str,
    mod_id: u64,
    file_id: u64,
    key: Option<&str>,
    expires: Option<u64>,
) -> String {
    let mut url = format!(
        "https://api.nexusmods.com/v1/games/{domain}/mods/{mod_id}/files/{file_id}/download_link.json"
    );
    if let (Some(key), Some(expires)) = (key, expires) {
        url.push_str(&format!("?key={key}&expires={expires}"));
    }
    url
}

pub async fn fetch_download_links(
    domain: &str,
    mod_id: u64,
    file_id: u64,
    api_key: Option<&str>,
    key: Option<&str>,
    expires: Option<u64>,
) -> AppResult<Vec<String>> {
    let url = build_download_link_url(domain, mod_id, file_id, key, expires);
    let client = reqwest::Client::new();
    let mut request = client
        .get(&url)
        .header("Application-Name", "Supervisor")
        .header("Application-Version", env!("CARGO_PKG_VERSION"));
    if let Some(api_key) = api_key.filter(|k| !k.trim().is_empty()) {
        request = request.header("apikey", api_key);
    }
    let response = request.send().await?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::user(
            "Invalid Nexus API key. Add a valid key in Settings (from nexusmods.com/users/myaccount?tab=api).",
        ));
    }
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(AppError::user(
            "Nexus Mods denied download access. Premium users need an API key; free users must use Mod Manager Download from the website.",
        ));
    }
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::user(format!(
            "Could not resolve download link from Nexus ({status}): {body}"
        )));
    }

    let body: ApiDownloadLinkResponse = response.json().await?;
    let mut urls = Vec::new();
    if let Some(link) = body.download_link.and_then(|l| l.uri) {
        urls.push(link);
    }
    for link in body.download_links {
        if let Some(uri) = link.uri {
            urls.push(uri);
        }
    }
    if urls.is_empty() {
        return Err(AppError::user(
            "Nexus returned no download URLs for this file. The mod may have been removed or the link expired.",
        ));
    }
    Ok(urls)
}

pub async fn validate_api_key(api_key: &str) -> AppResult<()> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.nexusmods.com/v1/users/validate.json")
        .header("apikey", api_key)
        .header("Application-Name", "Supervisor")
        .header("Application-Version", env!("CARGO_PKG_VERSION"))
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(AppError::user("Invalid Nexus API key."));
    }
    if !response.status().is_success() {
        return Err(AppError::user(format!(
            "Nexus API validation failed ({})",
            response.status()
        )));
    }
    Ok(())
}

pub async fn fetch_latest_file_version(
    domain: &str,
    mod_id: u64,
    file_id: u64,
    api_key: &str,
) -> AppResult<Option<String>> {
    let url =
        format!("https://api.nexusmods.com/v1/games/{domain}/mods/{mod_id}/files/{file_id}.json");
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("apikey", api_key)
        .header("Application-Name", "Supervisor")
        .header("Application-Version", env!("CARGO_PKG_VERSION"))
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let body: ApiFileResponse = response.json().await?;
    Ok(body.version.filter(|v| !v.trim().is_empty()))
}

pub fn normalize_version(value: &str) -> String {
    value.trim().to_lowercase()
}

/// Returns true when both sides have a version string and they differ after normalization.
pub fn versions_differ(installed: Option<&str>, remote: Option<&str>) -> bool {
    let Some(remote) = remote.filter(|v| !v.trim().is_empty()) else {
        return false;
    };
    let Some(installed) = installed.filter(|v| !v.trim().is_empty()) else {
        return false;
    };
    normalize_version(installed) != normalize_version(remote)
}

#[cfg(test)]
mod version_tests {
    use super::*;

    #[test]
    fn versions_match_after_normalization() {
        assert!(!versions_differ(Some("1.0.0"), Some("1.0.0 ")));
        assert!(!versions_differ(Some("v2"), Some("V2")));
    }

    #[test]
    fn missing_installed_version_is_not_an_update() {
        assert!(!versions_differ(None, Some("1.0.0")));
    }

    #[test]
    fn detects_real_version_change() {
        assert!(versions_differ(Some("1.0"), Some("1.1")));
    }
}
