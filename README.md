# hosthog

announce which resources you need on collaboratively used linux hosts

```
Usage: hosthog [COMMAND]

Commands:
  status   show current claims
  claim    Claim a resource. Fails if already claimed exclusively
  release  prematurely release a claim (removes all of your hogs and exclusive claims)
  hog      Hog the entire host (others will hate you)
  post     post a message to all logged in users
  users    List all logged in users
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

## Implementation status

- `hog` clears all AuthorizedKeysFiles via bind-mounting overlay files
- `release` restores all AuthorizedKeysFiles by unmounting bind-mounts
- `users` lists active users via `who`, and `netstat`
- `post` sends a message via `wall`
- `claim` hosthog maintains a list of claims which time out. You need an exclusive claim to hog the system.
- `status` lists claims


## Installation

Optional, but recommended dependencies: `at`

User-local installation via cargo: `cargo install --path .`

Or run it from within a nix shell: `nix shell .#default`
