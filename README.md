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
    },
    "TSX": {
      "language_servers": ["vtsls", "nudo", "..."]
    }
  },
  "code_lens": "on",
  "inlay_hints": { "enabled": true }
}
```

Notes:

- Zed's language names are `JavaScript` (`.js`/`.mjs`/`.jsx`), `TypeScript` (`.ts`),
  `TSX` (`.tsx`) — there are no `JavaScript React` / `TypeScript React` languages in Zed.
- `"..."` keeps every other registered server enabled, so once the extension is
  installed Zed also auto-includes `nudo` by default; listing it explicitly makes
  it the *primary* server, which is usually not what you want — keep `vtsls` (or
  `typescript-language-server`) primary and let `nudo` run as the secondary.
  Hover, diagnostics, inlay hints and code lenses from `nudo` are merged alongside
  the primary server's results.
- `nudo` analyzes `.js` / `.mjs` / `.ts` only (`isNudoTargetPath`); `.jsx` / `.tsx` /
  `.cjs` / `.d.ts` and sidecar contract modules are not inference targets.
- By default only files containing `@nudo:` directives are analyzed
  (`analysis.mode: "directives"`). To analyze every export, set
  `"nudo": { "analysis": { "mode": "exports" } }` in `package.json#nudo`
  (or `nudo.json`).

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

### Observation selector: contract vs cases

The lens row is an observation selector — **contract is one option alongside
each case**, exactly one active at a time (`●` active / `○` inactive):

- `● contract / hw|gen|imp` (handwritten / generated / implicit tier) — the
  default. Click to switch back to the contract observation
  (`nudo.selectContract`); clicking with no active case is an idempotent no-op.
- `○ case "name"` — click to select (`nudo.selectCase`); the contract option
  flips to `○`.
- `⚡ persist interface` / `⚡ draft interface` actions and the synthesized
  `call@L6 (5, 3) => 2` / `entry@` observation lenses stay visible regardless.

Hover and Abs type inlays (param constraints, return term/pred) always show —
they are the inference surface, not the observation vocabulary. The inlay tier
projection (for clients without CodeLens UI) mirrors the active state.

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
   version = "0.1.3"
   ```

4. Run `pnpm sort-extensions`, then open the PR.

5. Bump `version` in `extension.toml` and push when releasing updates.

## License

MIT — see [LICENSE](./LICENSE).
