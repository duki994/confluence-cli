# Authentication

> **v0.1 status:** the auth flow described here is the **planned shape**;
> it lands in milestone M1. The current M0 build returns
> `not yet implemented` for every `auth` subcommand.

## API tokens (Atlassian Cloud)

Confluence Cloud authentication in v0.1 uses Atlassian API tokens. Generate
one at <https://id.atlassian.com/manage-profile/security/api-tokens>.

```sh
confluence auth login
```

You will be prompted for:

1. The site URL (e.g. `your-org.atlassian.net`).
2. Your Atlassian account email.
3. The API token from the link above.

The CLI validates the credential against `/wiki/rest/api/user/current` and,
on success, stores the token in your operating system's keyring:

- **macOS:** Keychain
- **Windows:** Credential Manager
- **Linux:** Secret Service (GNOME Keyring, KWallet, …)

Host metadata (URL, email, auth method) is written to
`~/.config/confluence/hosts.toml`. The token itself **never** appears on
disk in cleartext.

## Multiple hosts

You can be logged into more than one host simultaneously:

```sh
confluence auth login                              # add a new host
confluence auth status                             # show all configured hosts
confluence auth switch your-other-org.atlassian.net
confluence --host your-other-org.atlassian.net page view 123
```

Per-invocation `--host` overrides the active default.

## Headless / CI environments

The OS keyring requires a desktop session on Linux. For CI or headless
servers, an encrypted file fallback is planned (M1+). Track the
`confluence-auth` crate for status.
