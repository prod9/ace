# Authentication

ACE handles OAuth token acquisition for services declared in the school's `[[auth]]` config.
All flows use PKCE (Proof Key for Code Exchange) exclusively.

## Why PKCE

Standard OAuth authorization code flow requires a `client_secret` on the token exchange. For a
CLI tool distributed to developer machines, there is no safe place to store that secret — it
would be embedded in the binary or config and trivially extractable.

PKCE eliminates the client secret. Instead, each auth session generates a one-time
cryptographic proof that binds the token exchange to the same client that initiated the request.

## What PKCE Protects Against

PKCE prevents **authorization code interception**: a malicious app on the same machine
intercepts the redirect callback and steals the authorization code. Without the `code_verifier`
(which never leaves the original client), the stolen code is useless.

PKCE does **not** protect against a full MITM that can intercept both the outgoing authorization
request and the incoming callback. TLS on all endpoints is assumed.

## Flow

1. ACE generates a random `code_verifier` (high-entropy string).
2. ACE computes `code_challenge = BASE64URL(SHA256(code_verifier))`.
3. ACE opens the browser to the service's `authorize_url` with:
   - `client_id`
   - `code_challenge`
   - `code_challenge_method=S256`
   - `scope` (from `[[auth]]` config)
   - `redirect_uri=http://localhost:{port}/callback`
   - `state` (random, to prevent CSRF)
4. ACE starts a temporary HTTP server on localhost to receive the callback.
5. User authorizes in the browser. Service redirects to the localhost callback with an
   `authorization_code` and `state`.
6. ACE verifies `state` matches, then exchanges the `authorization_code` + `code_verifier` at
   the service's `token_url` for an access token.
7. ACE stores the token in `~/.config/ace/config.toml` under `context.*.tokens.<name>`.
8. Localhost server shuts down.

## When It Runs

- **First run** — ACE prompts for all services declared in `[[auth]]` that have no stored token.
- **On template resolution failure** — if `{{ tokens.<name> }}` cannot resolve, ACE offers to
  re-authenticate. If the user declines, ACE warns and skips the MCP server that needed it.
- **Manual** — `ace auth <name>` to re-authenticate a specific service (e.g. token expired).

## Token Storage

Tokens are stored in the user-level config (`~/.config/ace/config.toml`) under
`context.*.tokens.<name>`, where `<name>` matches the `[[auth]]` entry's `name` field.

```toml
[context.acme.tokens]
github = "gho_..."
jira = "eyJ..."
```

Tokens must never appear in git-committed files (`ace.toml`, `school.toml`). ACE will error and
refuse to start if it detects token values in committed config. The school declares *which*
services exist and their OAuth endpoints; the user's machine holds the actual credentials.
