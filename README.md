# hosthog
announce which resources you need on collaboratively used linux hosts

# Design decisions

### Why no google sheets?

Google docs APIs are built for GUI applications instead of for server daemons. 
It wants to authenticate humans and not machines. 
That means authentication may last an arbitrarily short time and relies on browser interaction.
Therefore, it is badly suited for a tool that should be easy to use from the commandline without additional setup (after an admin has set it up). 

Google: Learn about authentication & authorization: [Auth Overivew](https://developers.google.com/workspace/guides/auth-overview)

> Go through the described steps to obtain an API key and enable the Sheets API for your account:
> 
> https://developers.google.com/workspace/guides/create-project
