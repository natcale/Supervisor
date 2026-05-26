/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use super::types::{NxmModLink, NxmOAuthCallback, NxmPayload};
use url::Url;

pub fn parse_nxm_url(raw: &str) -> Option<NxmPayload> {
    let trimmed = raw.trim().trim_matches('"');
    let url = Url::parse(trimmed).ok()?;

    if url.scheme() != "nxm" {
        return None;
    }

    let host = url.host_str().unwrap_or("").to_lowercase();
    let path = url.path();

    if host == "oauth" && path.starts_with("/callback") {
        let code = url
            .query_pairs()
            .find(|(k, _)| k == "code")
            .map(|(_, v)| v.to_string())?;
        let state = url
            .query_pairs()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.to_string());
        return Some(NxmPayload::OAuthCallback(NxmOAuthCallback { code, state }));
    }

    let segments: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    // nxm://{domain}/mods/{modId}/files/{fileId}
    if segments.len() >= 4 && segments[0] == "mods" && segments[2] == "files" {
        let game_domain = if host.is_empty() {
            return Some(NxmPayload::Unknown {
                raw: trimmed.to_string(),
            });
        } else {
            host
        };
        let mod_id = segments[1].parse().ok()?;
        let file_id = segments[3].parse().ok()?;
        return Some(build_mod_link(&url, game_domain, mod_id, file_id));
    }

    // nxm:///{domain}/mods/{modId}/files/{fileId} (domain in path)
    if segments.len() >= 5 && segments[1] == "mods" && segments[3] == "files" {
        let game_domain = segments[0].to_lowercase();
        let mod_id = segments[2].parse().ok()?;
        let file_id = segments[4].parse().ok()?;
        return Some(build_mod_link(&url, game_domain, mod_id, file_id));
    }

    Some(NxmPayload::Unknown {
        raw: trimmed.to_string(),
    })
}

fn build_mod_link(url: &Url, game_domain: String, mod_id: u64, file_id: u64) -> NxmPayload {
    let key = url
        .query_pairs()
        .find(|(k, _)| k == "key")
        .map(|(_, v)| v.to_string());
    let expires = url
        .query_pairs()
        .find(|(k, _)| k == "expires")
        .and_then(|(_, v)| v.parse().ok());
    let user_id = url
        .query_pairs()
        .find(|(k, _)| k == "user_id")
        .and_then(|(_, v)| v.parse().ok());

    NxmPayload::ModDownload(NxmModLink {
        game_domain,
        mod_id,
        file_id,
        key,
        expires,
        user_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mod_link_with_optional_params() {
        let payload = parse_nxm_url("nxm://Skyrim/mods/3863/files/1000172397").unwrap();
        match payload {
            NxmPayload::ModDownload(link) => {
                assert_eq!(link.game_domain, "skyrim");
                assert_eq!(link.mod_id, 3863);
                assert_eq!(link.file_id, 1000172397);
            }
            _ => panic!("expected mod link"),
        }
    }

    #[test]
    fn parses_mod_link_with_legacy_key() {
        let payload = parse_nxm_url(
            "nxm://skyrim/mods/3863/files/1000172397?key=abc&expires=1234567890&user_id=42",
        )
        .unwrap();
        match payload {
            NxmPayload::ModDownload(link) => {
                assert_eq!(link.key.as_deref(), Some("abc"));
                assert_eq!(link.expires, Some(1234567890));
                assert_eq!(link.user_id, Some(42));
            }
            _ => panic!("expected mod link"),
        }
    }

    #[test]
    fn parses_oauth_callback() {
        let payload = parse_nxm_url("nxm://oauth/callback?code=secret-code&state=xyz").unwrap();
        match payload {
            NxmPayload::OAuthCallback(cb) => {
                assert_eq!(cb.code, "secret-code");
                assert_eq!(cb.state.as_deref(), Some("xyz"));
            }
            _ => panic!("expected oauth callback"),
        }
    }
}
