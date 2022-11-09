# hosthog
announce which resources you need on collaboratively used linux hosts

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
  

### Why no google sheets?

Google docs APIs are built for GUI applications instead of for server daemons. 
It wants to authenticate humans and not machines. 
That means authentication may last an arbitrarily short time and relies on browser interaction.
Therefore, it is badly suited for a tool that should be easy to use from the commandline without additional setup (after an admin has set it up). 

Google: Learn about authentication & authorization: [Auth Overivew](https://developers.google.com/workspace/guides/auth-overview)

> Go through the described steps to obtain an API key and enable the Sheets API for your account:
> 
> https://developers.google.com/workspace/guides/create-project
