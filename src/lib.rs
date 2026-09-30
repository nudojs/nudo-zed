use std::{env, fs};
use zed_extension_api::{
    self as zed, serde_json, settings::LspSettings, LanguageServerId, Result,
};

const PACKAGE_NAME: &str = "@nudojs/lsp";
const SERVER_PATH: &str = "node_modules/@nudojs/lsp/dist/server.js";
const LANGUAGE_SERVER_ID: &str = "nudo";

struct NudoExtension {
    did_find_server: bool,
}

impl NudoExtension {
    fn server_exists(&self) -> bool {
        fs::metadata(SERVER_PATH).is_ok_and(|stat| stat.is_file())
    }

    fn server_script_path(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        if self.did_find_server && self.server_exists() {
            return Ok(SERVER_PATH.to_string());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let version = zed::npm_package_latest_version(PACKAGE_NAME)?;

        if !self.server_exists()
            || zed::npm_package_installed_version(PACKAGE_NAME)?.as_ref() != Some(&version)
        {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            let result = zed::npm_install_package(PACKAGE_NAME, &version);
            match result {
                Ok(()) => {
                    if !self.server_exists() {
                        Err(format!(
                            "installed package '{PACKAGE_NAME}' did not contain expected path '{SERVER_PATH}'",
                        ))?;
                    }
                }
                Err(error) => {
                    if !self.server_exists() {
                        Err(error)?;
                    }
                }
            }
        }

        self.did_find_server = true;
        Ok(SERVER_PATH.to_string())
    }
}

impl zed::Extension for NudoExtension {
    fn new() -> Self {
        Self {
            did_find_server: false,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        match language_server_id.as_ref() {
            LANGUAGE_SERVER_ID => {
                // User binary override: `lsp.nudo.binary` (path/arguments/env).
                let binary = LspSettings::for_worktree(LANGUAGE_SERVER_ID, worktree)
                    .ok()
                    .and_then(|settings| settings.binary);
                if let Some(binary) = binary {
                    if let Some(command) = binary.path {
                        return Ok(zed::Command {
                            command,
                            args: binary
                                .arguments
                                .unwrap_or_else(|| vec!["--stdio".to_string()]),
                            env: binary.env.unwrap_or_default().into_iter().collect(),
                        });
                    }
                }

                let server_path = self.server_script_path(language_server_id)?;
                Ok(zed::Command {
                    command: zed::node_binary_path()?,
                    args: vec![
                        env::current_dir()
                            .map_err(|e| e.to_string())?
                            .join(&server_path)
                            .to_string_lossy()
                            .to_string(),
                        "--stdio".to_string(),
                    ],
                    env: Default::default(),
                })
            }
            language_server_id => Err(format!("unknown language server: {language_server_id}")),
        }
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        LspSettings::for_worktree(LANGUAGE_SERVER_ID, worktree)
            .map(|settings| settings.initialization_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        LspSettings::for_worktree(LANGUAGE_SERVER_ID, worktree).map(|settings| settings.settings)
    }
}

zed::register_extension!(NudoExtension);
