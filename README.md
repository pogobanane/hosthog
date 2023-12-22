# hosthog

announce which resources you need on collaboratively used linux hosts

```
Usage: hosthog [COMMAND]

Commands:
  status   show current claims
  claim    Claim a resource. Fails if already claimed exclusively
  release  prematurely release a claim
  hog      Hog the entire host (others will hate you)
  post     post a message to all logged in users
  users    List all logged in users
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help information
  -V, --version  Print version information
```

## Implementation status

- `hog` clears all AuthorizedKeysFiles via bind-mounting /dev/null
- `release` restores all AuthorizedKeysFiles by unmounting bind-mounts
- `users` lists active users via `who`, and `netstat`
- `post` sends a message via `wall`
- `claim` hosthog maintains a list of claims, but doesn't really do anything with it yet
- `status` lists claims

