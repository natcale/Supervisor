/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
//! Zero-copy VDF parser for Steam libraryfolders and appmanifest files.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VdfEntry<'a> {
    pub key: &'a str,
    pub value: VdfValue<'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VdfValue<'a> {
    String(&'a str),
    Object(Vec<VdfEntry<'a>>),
}

pub fn parse_vdf(input: &str) -> Result<Vec<VdfEntry<'_>>, VdfError> {
    let bytes = input.as_bytes();
    let mut pos = 0;
    skip_ws(bytes, &mut pos);
    parse_object(bytes, &mut pos)
}

#[derive(Debug, thiserror::Error)]
pub enum VdfError {
    #[error("unexpected end of VDF input")]
    UnexpectedEof,

    #[error("invalid VDF token at byte {0}")]
    InvalidToken(usize),

    #[error("unclosed VDF object")]
    #[allow(dead_code)]
    UnclosedObject,
}
fn skip_ws(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() {
        match bytes[*pos] {
            b' ' | b'\t' | b'\r' | b'\n' => *pos += 1,
            b'/' if *pos + 1 < bytes.len() && bytes[*pos + 1] == b'/' => {
                *pos += 2;
                while *pos < bytes.len() && bytes[*pos] != b'\n' {
                    *pos += 1;
                }
            }
            _ => break,
        }
    }
}

fn read_quoted<'a>(bytes: &'a [u8], pos: &mut usize) -> Result<&'a str, VdfError> {
    skip_ws(bytes, pos);
    if *pos >= bytes.len() || bytes[*pos] != b'"' {
        return Err(VdfError::InvalidToken(*pos));
    }
    *pos += 1;
    let start = *pos;
    while *pos < bytes.len() {
        if bytes[*pos] == b'\\' {
            *pos += 2;
            continue;
        }
        if bytes[*pos] == b'"' {
            let end = *pos;
            *pos += 1;
            let s = std::str::from_utf8(&bytes[start..end])
                .map_err(|_| VdfError::InvalidToken(start))?;
            return Ok(s);
        }
        *pos += 1;
    }
    Err(VdfError::UnexpectedEof)
}

fn parse_object<'a>(bytes: &'a [u8], pos: &mut usize) -> Result<Vec<VdfEntry<'a>>, VdfError> {
    let mut entries = Vec::new();
    loop {
        skip_ws(bytes, pos);
        if *pos >= bytes.len() {
            break;
        }
        if bytes[*pos] == b'}' {
            *pos += 1;
            break;
        }

        let key = read_quoted(bytes, pos)?;
        skip_ws(bytes, pos);

        if *pos >= bytes.len() {
            return Err(VdfError::UnexpectedEof);
        }

        let value = if bytes[*pos] == b'{' {
            *pos += 1;
            VdfValue::Object(parse_object(bytes, pos)?)
        } else {
            VdfValue::String(read_quoted(bytes, pos)?)
        };

        entries.push(VdfEntry { key, value });
    }
    Ok(entries)
}

pub fn find_string<'a>(entries: &'a [VdfEntry<'a>], key: &str) -> Option<&'a str> {
    entries.iter().find_map(|entry| {
        if entry.key == key {
            if let VdfValue::String(v) = entry.value {
                return Some(v);
            }
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::{find_string, parse_vdf, VdfValue};

    #[test]
    fn parses_libraryfolders() {
        let input = r#"
"libraryfolders"
{
    "0"
    {
        "path"        "C:\\SteamLibrary"
        "apps"
        {
            "72850"        "1"
        }
    }
}
"#;
        let parsed = parse_vdf(input).unwrap();
        let root = parsed
            .first()
            .and_then(|e| {
                if let VdfValue::Object(ref obj) = e.value {
                    Some(obj.as_slice())
                } else {
                    None
                }
            })
            .unwrap();
        let folder0 = root
            .iter()
            .find(|e| e.key == "0")
            .and_then(|e| {
                if let VdfValue::Object(ref obj) = e.value {
                    Some(obj.as_slice())
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(find_string(folder0, "path"), Some("C:\\\\SteamLibrary"));
    }
}
