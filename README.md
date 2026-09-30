# nudo — Zed extension

[Zed](https://zed.dev) extension that registers the [Nudo](https://github.com/nudojs/nudo)
language server (`@nudojs/lsp`) as a **secondary** language server for JavaScript and TypeScript
(alongside `vtsls` / `typescript-language-server`).

Requires [`@nudojs/lsp` ≥ 0.5.0](https://www.npmjs.com/package/@nudojs/lsp) — that release ships the
`nudo-lsp` bin and defaults to stdio when no transport flag is passed.

## Install

Once published to the [Zed extensions registry](https://github.com/zed-industries/extensions):
**Extensions → search “nudo-lsp” → Install**.

### Dev install (from this repo)

```bash
rustup target add wasm32-wasip2   # once
```

In Zed: **Extensions → Install Dev Extension…** → select this repository root.

### Language server

The extension installs the latest `@nudojs/lsp` via Zed’s managed npm
(`npm_install_package`) and launches `node_modules/@nudojs/lsp/dist/server.js --stdio`.
No manual install is required.

### Settings

```json
{
  "languages": {
    "JavaScript": {
      "language_servers": ["vtsls", "nudo", "..."]
    },
    "TypeScript": {
      "language_servers": ["vtsls", "nudo", "..."]
    }
  },
  "code_lens": "on",
  "inlay_hints": { "enabled": true }
}
```

Enable semantic tokens if you want type-aware highlighting:

```json
{
  "semantic_tokens": "combined"
}
```

### LSP settings passthrough

Like other Zed language-server extensions (and the VS Code client settings face),
`lsp.nudo.settings` / `lsp.nudo.initialization_options` are forwarded to the server:

```json
{
  "lsp": {
    "nudo": {
      "settings": {},
      "initialization_options": {}
    }
  }
}
```

Project analysis config still lives in `package.json#nudo` (or `nudo.json`) —
`analysis.mode` / `include` / `exclude` are project config, not editor config.

## How the server is found

The extension resolves the server the same way as other Zed language-server
extensions (e.g. Astro): `npm_install_package("@nudojs/lsp")` into the
extension work directory, then run

```
<node_binary_path>/node_modules/@nudojs/lsp/dist/server.js --stdio
```

Binary override (skip the managed install):

```json
{
  "lsp": {
    "nudo": {
      "binary": {
        "path": "nudo-lsp",
        "arguments": []
      }
    }
  }
}
```

Or point at a project install:

```json
{
  "lsp": {
    "nudo": {
      "binary": {
        "path": "node",
        "arguments": ["/abs/path/node_modules/@nudojs/lsp/dist/server.js"]
      }
    }
  }
}
```

## Build

Zed compiles the extension (WASM, `wasm32-wasip2`) when you install a dev extension.
Manual check:

```bash
cargo build --target wasm32-wasip2 --release
```

## Publish to the Zed registry

Zed installs extensions as git submodules under
[zed-industries/extensions](https://github.com/zed-industries/extensions).

1. Push this repo to GitHub (`nudojs/nudo-zed`).
2. Open a PR against `zed-industries/extensions`:

   ```bash
   git submodule add https://github.com/nudojs/nudo-zed extensions/nudo-lsp
   git commit -m "Add nudo-lsp extension"
   ```

3. Add to `extensions.toml`:

   ```toml
   [nudo-lsp]
   submodule = "extensions/nudo-lsp"
   version = "0.1.2"
   ```

4. Run `pnpm sort-extensions`, then open the PR.

5. Bump `version` in `extension.toml` and push when releasing updates.

## License

MIT — see [LICENSE](./LICENSE).
