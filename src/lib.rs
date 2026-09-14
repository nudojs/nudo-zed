use zed_extension_api::{
    self as zed, Command, LanguageServerId, Result, Worktree, process,
};

const PACKAGE: &str = "@nudojs/lsp";
const SERVER_REL: &str = "node_modules/@nudojs/lsp/dist/server.js";
const BIN_NAME: &str = "nudo-lsp";

struct NudoExtension {
    cached_server_path: Option<String>,
}

impl NudoExtension {
    fn shell_env(worktree: &Worktree) -> Vec<(String, String)> {
        worktree.shell_env()
    }

    fn node_command(node: &str, server_path: String, worktree: &Worktree) -> Command {
        Command {
            command: node.to_string(),
            args: vec![server_path, "--stdio".to_string()],
            env: Self::shell_env(worktree),
        }
    }

    /// Resolve the absolute path of the npm-installed server via Node itself.
    /// `npm_install_package` drops packages in Zed's extension work dir; the
    /// host process has that on NODE_PATH, so `require.resolve` finds them.
    fn resolve_via_node(node: &str) -> Result<String> {
        let mut cmd = process::Command::new(node)
            .arg("-p")
            .arg("require.resolve('@nudojs/lsp')");
        let output = cmd
            .output()
            .map_err(|e| format!("failed to run node require.resolve: {e}"))?;
        if output.status != Some(0) {
            return Err(format!(
                "require.resolve('@nudojs/lsp') failed (status {:?}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            return Err("require.resolve('@nudojs/lsp') returned empty path".into());
        }
        Ok(path)
    }

    fn ensure_npm_package() -> Result<()> {
        let installed = zed::npm_package_installed_version(PACKAGE)?;
        let latest = zed::npm_package_latest_version(PACKAGE)
            .map_err(|e| format!("cannot query npm for {PACKAGE}: {e}"))?;
        if installed.as_deref() != Some(latest.as_str()) {
            zed::npm_install_package(PACKAGE, &latest)
                .map_err(|e| format!("npm install {PACKAGE}@{latest} failed: {e}"))?;
        }
        Ok(())
    }

    fn resolve_command(&mut self, worktree: &Worktree) -> Result<Command> {
        // 1. Global `nudo-lsp` bin (npm i -g @nudojs/lsp)
        if let Some(bin) = worktree.which(BIN_NAME) {
            return Ok(Command {
                command: bin,
                args: vec!["--stdio".to_string()],
                env: Self::shell_env(worktree),
            });
        }

        // 2. Project-local install (preferred for version-pinned projects)
        if worktree.read_text_file(SERVER_REL).is_ok() {
            let path = format!("{}/{}", worktree.root_path(), SERVER_REL);
            self.cached_server_path = Some(path.clone());
            let node = worktree.which("node").ok_or_else(|| {
                "node not found on PATH; install Node.js or put nudo-lsp on PATH".to_string()
            })?;
            return Ok(Self::node_command(&node, path, worktree));
        }

        // 3. Zed-managed npm install, resolved via require.resolve
        if let Some(cached) = self.cached_server_path.clone() {
            let node = zed::node_binary_path()
                .map_err(|e| format!("Zed Node runtime unavailable: {e}"))?;
            // Cache is only reused when resolve still points at it; otherwise reinstall.
            if Self::resolve_via_node(&node).ok().as_deref() == Some(cached.as_str()) {
                return Ok(Self::node_command(&node, cached, worktree));
            }
        }

        let node = zed::node_binary_path()
            .map_err(|e| format!("Zed Node runtime unavailable: {e}"))?;
        Self::ensure_npm_package()?;
        let path = Self::resolve_via_node(&node)?;
        self.cached_server_path = Some(path.clone());
        Ok(Self::node_command(&node, path, worktree))
    }
}

impl zed::Extension for NudoExtension {
    fn new() -> Self {
        Self {
            cached_server_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        self.resolve_command(worktree)
    }
}

zed::register_extension!(NudoExtension);
