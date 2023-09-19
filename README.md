# hosthog
announce which resources you need on collaboratively used linux hosts

# Status

in development/stale, is not even a prototype

# Design decisions

### Use-cases

A user logs into a server he as been using regularly. 
At login-time he wants to be prompted with a message, describing who is using the same server right now. 
This way, he can notice, if a new person started using the server - without having to remember to check some external resource. 
(message of the day)

A user is working on a server. He wants to be prompted with a message, when someone declares a new use of the local host. 
This way, he can notice, if someone else needs exclusive access to a resource the user is using right now. 
(wall)

A user wants to quickly reboot a machine. He wants a console command to claim exclusive access in a few minutes for a few minutes. 

A user wants to reserve hardware for a long running research project running till a deadline. 

A user wants to edit or extend a reservation.

A user wants to query, which users are online right now (`users`, running processes, ...)

A user might want to use `hosthog` infrastructure to notify other users.

An admin wants to leave a message to notify users of a host about notable nixos config changes. 

### Architcture

- Per-host daemon/service
  - DB with reservations for localhost
  - updates message of the day with reservations for localhost for today
  - posts updates via wall to logged in users
  - syncs reservations with a (~google~ nextcloud, see "Why no google sheets" section) calendar
- User cli client
  - used to add/change/view localhost reservations
- optional: State server
  - global reservation DB for cluster usage statistics
  - is notified by host daemons about changes
  - exposes reservations via website (calendar?)
  - (pushes reservation events to client daemons)
- optional: google calendar being filled with events by host daemons


### Implementation

#### Push notifications in console: 

- pseudo terminals: sudo wall
- tmux sessions: tmux display-popup  
  ```
  # for each user:
  tmux lsc
  tmux display-popup -c /dev/pts/0 "echo message && read"
  ```
- screen: receives walls
- xrdp: ?
- vscode remote plugin: ?

Message of the day:

is not displayed in tmux


#### Locking down a host (hogging)

`hosthog hog`

- only allowed when holding an exclusive claim
- has a global setting: AuthorizedKeysFile. Specifies the location of said files (see man sshd\_config).
- bind-mounts empty files over all users except root and the current one
- has optional argument to allow other user sets
- keeps disk state about which mounts it has active
- has optional arguemnt to unhog
- unhogs automatically, once the exclusive claim expires


#### google sheets/calendar?

Google api client: either rust impls (see Cargo.toml) or some cli tool like [gcalcli](https://github.com/insanum/gcalcli).

Google docs APIs are built for GUI applications instead of for server daemons. 
It wants to authenticate humans and not machines. 
That means authentication may last an arbitrarily short time and relies on browser interaction.
Therefore, it is badly suited for a tool that should be easy to use from the commandline without additional setup (after an admin has set it up). 

Google: Learn about authentication & authorization: [Auth Overivew](https://developers.google.com/workspace/guides/auth-overview)

> Go through the described steps to obtain an API key and enable the Sheets API for your account:
> 
> https://developers.google.com/workspace/guides/create-project

Or is it badly suited?

There are google service accounts however that can be created in your google cloud project -> IAM -> Service-accounts.
These can be used for bot login.

That gives you a credentials.json that the official libraries know how to use https://developers.google.com/sheets/api/quickstart/python

And then use it with https://crates.io/crates/google-authz

#### Notes

notifications in x:

`DBUS_SESSION_BUS_ADDRESS=/run/user/1000/bus notify-send 'test message'`

https://discourse.nixos.org/t/desktop-notifications-from-systemd-service/17672/6

detecting vscode users: 
joerg      94310  0.0  0.0   2888   724 ?        Ss   15:54   0:00 sh /home/dimitra/.vscode-server/bin/6261075646f055b99068d3688932416f2346dd3b/bin/code-server
