/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use crate::errors::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;

pub fn bg3_mods_dir() -> Option<PathBuf> {
    local_app_data().map(|base| {
        base.join("Larian Studios")
            .join("Baldur's Gate 3")
            .join("Mods")
    })
}

pub fn bg3_modsettings_path() -> Option<PathBuf> {
    local_app_data().map(|base| {
        base.join("Larian Studios")
            .join("Baldur's Gate 3")
            .join("PlayerProfiles")
            .join("Public")
            .join("modsettings.lsx")
    })
}

pub fn sync_modsettings(pak_mod_names: &[String]) -> AppResult<()> {
    if pak_mod_names.is_empty() {
        return Ok(());
    }
    let Some(path) = bg3_modsettings_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::Io)?;
    }

    let mut content = if path.is_file() {
        fs::read_to_string(&path).map_err(AppError::Io)?
    } else {
        default_modsettings_template()
    };

    for name in pak_mod_names {
        let module_name = pak_module_name(name);
        if content.contains(&format!("Name=\"{module_name}\"")) {
            continue;
        }
        content = insert_module_entry(&content, &module_name);
    }

    fs::write(&path, content).map_err(AppError::Io)?;
    Ok(())
}

fn pak_module_name(slug: &str) -> String {
    let base = slug.trim_end_matches(".pak");
    if base.ends_with("_pak") {
        base.to_string()
    } else {
        format!("{base}_pak")
    }
}

fn default_modsettings_template() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<save>
    <version major="4" minor="0" revision="9" build="328"/>
    <region id="ModuleSettings">
        <node id="root">
            <children>
                <node id="Mods">
                    <children/>
                </node>
            </children>
        </node>
    </region>
</save>
"#
    .to_string()
}

fn insert_module_entry(content: &str, module_name: &str) -> String {
    let entry = format!(
        r#"                <node id="Module">
                    <attribute id="UUID" type="FixedString" value="{}"/>
                    <attribute id="Name" type="LSString" value="{}"/>
                    <attribute id="Folder" type="LSString" value="{}"/>
                    <attribute id="MD5" type="LSString" value=""/>
                    <attribute id="PublishHandle" type="uint64" value="0"/>
                    <attribute id="Version64" type="int64" value="36028797018963968"/>
                </node>
"#,
        uuid::Uuid::new_v4(),
        module_name,
        module_name
    );

    if let Some(idx) = content.find("<children/>") {
        let mut out = String::with_capacity(content.len() + entry.len() + 32);
        out.push_str(&content[..idx]);
        out.push_str("<children>\n");
        out.push_str(&entry);
        out.push_str("                    </children>");
        out.push_str(&content[idx + "<children/>".len()..]);
        return out;
    }

    if let Some(idx) = content.find(r#"<node id="Mods">"#) {
        if let Some(rel) = content[idx..].find("<children>") {
            let insert_at = idx + rel + "<children>".len();
            let mut out = String::with_capacity(content.len() + entry.len());
            out.push_str(&content[..insert_at]);
            out.push('\n');
            out.push_str(&entry);
            out.push_str(&content[insert_at..]);
            return out;
        }
    }

    content.to_string()
}

fn local_app_data() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_module_into_empty_modsettings() {
        let base = default_modsettings_template();
        let updated = insert_module_entry(&base, "MyMod_pak");
        assert!(updated.contains("value=\"MyMod_pak\""));
    }
}
