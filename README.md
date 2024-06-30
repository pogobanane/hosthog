# 🦔 hosthog

Announce which resources you need on collaboratively used linux hosts.
Keep other processes away while you have an exclusive lock on the host.

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

Example:
```bash
sudo hosthog claim --exclusive 15min some benchmarks
sudo hosthog hog
```
For 15 minutes other users will be locked out from ssh and tasks scheduled by systemd are paused.

## Implementation status

- `claim` hosthog maintains a list of claims which time out. You need an exclusive claim to hog the system.
- `hog`: prevent things from happening that are not related to you
  - Clears all AuthorizedKeysFiles via bind-mounting overlay files. Locked out users receive a hosthog message when they attempt to connect via ssh.
  - Stops all systemd.timers.
- `release` releases exclusive claims and reverts `hog`
- `users` lists active users via `who`, and ssh sessions with `netstat`
- `post` sends a message via `wall`
- `status` lists claims


## Installation

Optional, but recommended dependencies: `at` (needed to remove claims on timeout)

User-local installation via cargo: `cargo install --path .`

Or run it from within a nix shell: `nix shell github:pogobanane/hosthog#default` (timeouts won't work)
