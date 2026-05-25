/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
mod api;
mod queue;

pub use api::*;
pub use queue::*;

use crate::deep_link::NxmModLink;
use crate::errors::{AppError, AppResult};
use futures_util::StreamExt;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub async fn download_mod_archive(
    link: &NxmModLink,
    dest_dir: &Path,
    api_key: Option<&str>,
    speed_limit_kbps: Option<u32>,
) -> AppResult<PathBuf> {
    fs::create_dir_all(dest_dir).map_err(AppError::Io)?;

    let download_url = resolve_download_url(link, api_key).await?;

    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("Application-Name", "Supervisor")
        .header("Application-Version", env!("CARGO_PKG_VERSION"))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::user(format!(
            "Nexus Mods declined the download ({status}): {body}. The link may have expired — \
             download again from the Nexus website."
        )));
    }

    let file_name = response
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .and_then(parse_content_disposition)
        .unwrap_or_else(|| format!("mod-{}-{}.zip", link.mod_id, link.file_id));

    let dest = dest_dir.join(&file_name);
    let mut file = tokio::fs::File::create(&dest).await.map_err(AppError::Io)?;
    let mut stream = response.bytes_stream();

    let max_bytes_per_sec = speed_limit_kbps.map(|kbps| (kbps as u64).saturating_mul(1024) / 8);
    let mut window_start = std::time::Instant::now();
    let mut window_bytes: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(AppError::Http)?;
        file.write_all(&chunk).await.map_err(AppError::Io)?;

        if let Some(limit) = max_bytes_per_sec {
            window_bytes += chunk.len() as u64;
            if window_bytes >= limit {
                let elapsed = window_start.elapsed();
                if elapsed.as_millis() < 1000 {
                    tokio::time::sleep(std::time::Duration::from_millis(1000 - elapsed.as_millis() as u64))
                        .await;
                }
                window_start = std::time::Instant::now();
                window_bytes = 0;
            }
        }
    }

    file.flush().await.map_err(AppError::Io)?;
    Ok(dest)
}

async fn resolve_download_url(link: &NxmModLink, api_key: Option<&str>) -> AppResult<String> {
    let api_key = api_key.filter(|k| !k.trim().is_empty());
    let nxm_key = link.key.as_deref().filter(|k| !k.is_empty());

    // Resolve via Nexus API (supports ephemeral NXM key+expires and premium API key).
    if nxm_key.is_some() || api_key.is_some() {
        if let Ok(urls) = fetch_download_links(
            &link.game_domain,
            link.mod_id,
            link.file_id,
            api_key,
            nxm_key,
            link.expires,
        )
        .await
        {
            if let Some(url) = urls.into_iter().next() {
                return Ok(url);
            }
        }
    }

    // Legacy direct CDN URL (often 404 when the NXM token expired).
    if let Some(key) = nxm_key {
        return Ok(build_keyed_download_url(link, key)?);
    }

    if api_key.is_some() {
        return Err(AppError::user(
            "Nexus Mods denied download access. Check your API key and premium status, or use Mod Manager Download from the website.",
        ));
    }

    Err(AppError::user(
        "Add your Nexus API key in Settings → Nexus Mods, or use Mod Manager Download from the Nexus website.",
    ))
}

fn build_keyed_download_url(link: &NxmModLink, key: &str) -> AppResult<String> {
    let expires = link
        .expires
        .map(|e| e.to_string())
        .unwrap_or_else(|| "0".into());
    let user_id = link
        .user_id
        .map(|u| u.to_string())
        .unwrap_or_else(|| "0".into());

    Ok(format!(
        "https://data.nexusmods.com/v1/games/{}/mods/{}/files/{}/{}/{}/{}",
        link.game_domain, link.mod_id, link.file_id, user_id, expires, key
    ))
}

fn parse_content_disposition(header: &str) -> Option<String> {
    for part in header.split(';') {
        let part = part.trim();
        if let Some(name) = part.strip_prefix("filename=") {
            return Some(name.trim_matches('"').to_string());
        }
        if let Some(name) = part.strip_prefix("filename*=") {
            let encoded = name.split('\'').last().unwrap_or(name);
            return Some(encoded.trim_matches('"').to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_download_link_url_with_key() {
        let url = api::build_download_link_url("marvelrivals", 8799, 123, Some("abc"), Some(999));
        assert!(url.contains("marvelrivals/mods/8799/files/123/download_link.json"));
        assert!(url.contains("key=abc"));
        assert!(url.contains("expires=999"));
    }

    #[test]
    fn build_download_link_url_premium() {
        let url = api::build_download_link_url("skyrim", 1, 2, None, None);
        assert_eq!(
            url,
            "https://api.nexusmods.com/v1/games/skyrim/mods/1/files/2/download_link.json"
        );
    }
}
