# hackthebot

A discord bot that announces the solves of a HTB team.

## Setup

1. Create a project [here](https://hasura.io/) or alternatively, host the Hasura instance yourself.
2. Install `hasura`
   cli ([refer](https://hasura.io/docs/latest/graphql/core/hasura-cli/install-hasura-cli.html#install-hasura-cli))
3. Store your Hasura admin token in the env var `HASURA_GRAPHQL_ADMIN_SECRET`
4. Clone repo and edit `migrations/config.yaml` to point to your Hasura instance.
5. Run the migrations as per below commands

    ```bash
    $ hasura migrate apply --all-databases --endpoint https://hackthebot.husk.cloud/
    
    $ hasura metadata reload --endpoint https://hackthebot.husk.cloud/
    ```
6. Browse `hasura console` and ensure the tables are tracked.
7. Grab a copy of the docker image available [here](https://github.com/Huskehhh/hackthebot/pkgs/container/hackthebot)

   Configuration is done via environment variables:
    ```env
    OWNER_ID=
    DISCORD_TOKEN=
    HTB_EMAIL=
    HTB_PASSWORD=
    HTB_TEAM_ID=
    HTB_CHANNEL_ID=
    HASURA_API_KEY=
    HASURA_API_URL=
    APPLICATION_ID=
    ```

## Commands

Available prefixes are ``!`` ``.`` ``~``

| Command                   | Description                       | Permission |
|---------------------------|-----------------------------------|------------|
| !htb search "*challenge*" | Searches for the given challenge  | CTFer      |
| !htb solves "*challenge*" | Searches for all solves of a user | CTFer      |
