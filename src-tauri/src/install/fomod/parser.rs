/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FomodOption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub option_type: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FomodStep {
    pub id: String,
    pub name: String,
    pub options: Vec<FomodOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FomodConfig {
    pub module_name: String,
    pub steps: Vec<FomodStep>,
}

pub fn parse_fomod_config(xml: &str) -> FomodConfig {
    let module_name = extract_tag_text(xml, "moduleName").unwrap_or_else(|| "Mod".into());

    let install_steps = extract_install_steps(xml);
    if !install_steps.is_empty() {
        return FomodConfig {
            module_name,
            steps: install_steps,
        };
    }

    let options = parse_config_options(xml);
    FomodConfig {
        module_name,
        steps: vec![FomodStep {
            id: "default".into(),
            name: "Installation options".into(),
            options,
        }],
    }
}

fn extract_install_steps(xml: &str) -> Vec<FomodStep> {
    let mut steps = Vec::new();
    for (idx, step_block) in extract_tags(xml, "installStep").into_iter().enumerate() {
        let name = extract_attr(&step_block, "name").unwrap_or_else(|| format!("Step {}", idx + 1));
        let mut options = Vec::new();

        for group in extract_tags(&step_block, "group") {
            options.extend(parse_group_plugins(&group));
        }
        for plugin in extract_tags(&step_block, "plugin") {
            options.push(parse_plugin_as_option(&plugin, options.len()));
        }

        if options.is_empty() {
            options.extend(parse_config_options(&step_block));
        }

        if !options.is_empty() {
            steps.push(FomodStep {
                id: format!("step-{idx}"),
                name,
                options,
            });
        }
    }
    steps
}

fn parse_group_plugins(group_xml: &str) -> Vec<FomodOption> {
    let mut options = Vec::new();
    for (idx, plugin) in extract_tags(group_xml, "plugin").into_iter().enumerate() {
        options.push(parse_plugin_as_option(&plugin, idx));
    }
    options
}

fn parse_plugin_as_option(plugin_xml: &str, idx: usize) -> FomodOption {
    let name = extract_attr(&plugin_xml, "name")
        .or_else(|| extract_tag_text(&plugin_xml, "name"))
        .unwrap_or_else(|| format!("Option {}", idx + 1));
    let description = extract_tag_text(&plugin_xml, "description").unwrap_or_default();
    FomodOption {
        id: format!("opt-{idx}"),
        name,
        description,
        option_type: "Optional".into(),
        is_default: idx == 0,
    }
}

fn parse_config_options(xml: &str) -> Vec<FomodOption> {
    let mut options = Vec::new();
    for (idx, block) in extract_tags(xml, "configOption").into_iter().enumerate() {
        let name = extract_attr(&block, "name").unwrap_or_else(|| format!("Option {}", idx + 1));
        let description = extract_tag_text(&block, "description").unwrap_or_default();
        let option_type = extract_attr(&block, "type").unwrap_or_else(|| "Optional".into());
        options.push(FomodOption {
            id: format!("opt-{idx}"),
            name,
            description,
            option_type,
            is_default: idx == 0,
        });
    }
    options
}

pub fn selected_option_blocks(xml: &str, selections: &[String]) -> Vec<String> {
    let config = parse_fomod_config(xml);
    let mut blocks = Vec::new();
    let flat_options = extract_tags(xml, "configOption");

    if !flat_options.is_empty() {
        for sel in selections {
            if let Some(idx) = sel
                .strip_prefix("opt-")
                .and_then(|s| s.parse::<usize>().ok())
            {
                if let Some(block) = flat_options.get(idx) {
                    blocks.push(block.clone());
                }
            }
        }
        return blocks;
    }

    let mut opt_idx = 0;
    for step in &config.steps {
        let sel = selections
            .iter()
            .find(|s| s.starts_with("step-") || s.starts_with("opt-"));
        let sel_id = sel.cloned().unwrap_or_else(|| format!("opt-{opt_idx}"));
        if let Some(idx) = sel_id
            .strip_prefix("opt-")
            .and_then(|s| s.parse::<usize>().ok())
        {
            let install_steps = extract_tags(xml, "installStep");
            if let Some(step_block) = install_steps.get(
                step.id
                    .strip_prefix("step-")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
            ) {
                let plugins: Vec<_> = extract_tags(step_block, "plugin");
                if let Some(plugin) = plugins.get(idx) {
                    blocks.push(plugin.clone());
                }
            }
        }
        opt_idx += step.options.len();
    }

    if blocks.is_empty() && !selections.is_empty() {
        if let Some(idx) = selections[0]
            .strip_prefix("opt-")
            .and_then(|s| s.parse::<usize>().ok())
        {
            if let Some(block) = flat_options.get(idx) {
                blocks.push(block.clone());
            }
        }
    }

    blocks
}

pub fn folder_mappings_from_blocks(blocks: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for block in blocks {
        for folder_block in extract_tags(block, "folder") {
            let source = extract_tag_text(&folder_block, "source").unwrap_or_default();
            let dest = extract_tag_text(&folder_block, "destination").unwrap_or_default();
            if !source.is_empty() {
                out.push((normalize_path(&source), normalize_path(&dest)));
            }
        }
        for file_block in extract_tags(block, "file") {
            let source = extract_tag_text(&file_block, "source").unwrap_or_default();
            let dest = extract_tag_text(&file_block, "destination").unwrap_or_default();
            if !source.is_empty() {
                out.push((normalize_path(&source), normalize_path(&dest)));
            }
        }
    }
    out
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string()
}

fn extract_tags(xml: &str, tag: &str) -> Vec<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find(&open) {
        let after_open = &rest[start..];
        let Some(end) = after_open.find(&close) else {
            break;
        };
        let end_idx = end + close.len();
        out.push(after_open[..end_idx].to_string());
        rest = &after_open[end_idx..];
    }
    out
}

fn extract_tag_text(block: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find(&close)? + start;
    Some(block[start..end].trim().to_string())
}

fn extract_attr(block: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=\"");
    let start = block.find(&needle)? + needle.len();
    let rest = &block[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config_options() {
        let xml = r#"<config>
            <moduleName>Test Mod</moduleName>
            <configOption name="Full">
                <folder><source>Data</source><destination>Data</destination></folder>
            </configOption>
            <configOption name="Lite">
                <folder><source>Lite</source><destination>Data</destination></folder>
            </configOption>
        </config>"#;
        let cfg = parse_fomod_config(xml);
        assert_eq!(cfg.module_name, "Test Mod");
        assert_eq!(cfg.steps[0].options.len(), 2);
    }
}
