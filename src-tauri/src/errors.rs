/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Supervisor contributors. All rights reserved.
 *  Licensed under the MIT License. See LICENSE in the project root for license information.
 *--------------------------------------------------------------------------------------------*/
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFacingIssue {
    pub id: String,
    pub title: String,
    pub explanation: String,
    pub impact: String,
    pub choices: Vec<UserChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserChoice {
    pub id: String,
    pub label: String,
    pub description: String,
    pub recommended: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    User(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Keyring(#[from] keyring::Error),

    #[error(transparent)]
    Url(#[from] url::ParseError),
}

impl AppError {
    pub fn user(message: impl Into<String>) -> Self {
        Self::User(message.into())
    }

    pub fn to_user_issue(&self) -> UserFacingIssue {
        match self {
            AppError::User(msg) => UserFacingIssue {
                id: "user-error".into(),
                title: "Something needs your attention".into(),
                explanation: msg.clone(),
                impact: "This action cannot continue until you resolve the issue.".into(),
                choices: vec![UserChoice {
                    id: "acknowledge".into(),
                    label: "Got it".into(),
                    description: "Return and try a different approach.".into(),
                    recommended: true,
                }],
            },
            AppError::Io(e) if e.kind() == std::io::ErrorKind::NotFound => UserFacingIssue {
                id: "missing-path".into(),
                title: "A required folder was not found".into(),
                explanation: format!(
                    "Supervisor looked for a file or folder that isn't there: {e}"
                ),
                impact: "The mod cannot be installed until the path exists.".into(),
                choices: vec![UserChoice {
                    id: "rescan".into(),
                    label: "Scan for games again".into(),
                    description: "Refresh detected installations and try again.".into(),
                    recommended: true,
                }],
            },
            AppError::Io(_) => UserFacingIssue {
                id: "io-error".into(),
                title: "Supervisor couldn't access a file".into(),
                explanation: self.to_string(),
                impact: "File access may be blocked by permissions or another program.".into(),
                choices: vec![UserChoice {
                    id: "retry".into(),
                    label: "Try again".into(),
                    description: "Close other programs using the files and retry.".into(),
                    recommended: true,
                }],
            },
            _ => UserFacingIssue {
                id: "unexpected".into(),
                title: "An unexpected problem occurred".into(),
                explanation: self.to_string(),
                impact: "Your mods were not changed.".into(),
                choices: vec![UserChoice {
                    id: "acknowledge".into(),
                    label: "Dismiss".into(),
                    description: "Return to the previous screen.".into(),
                    recommended: true,
                }],
            },
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
