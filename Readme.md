# Telegram bot

## Motivation and goals

* Learn rust
* Evaluate rust [teloxide](https://github.com/teloxide/teloxide) crate to write a telegram bot
* Production ready
  * Health checks
  * Self-hosted deployment options
* Features:
  * User registration (identify and recognize known users)
  * Broadcast messages to known users


## Getting started

### Create telegram bot 

* [Telegram Bot API Documentation](https://core.telegram.org/bots)
* Documentation on [Creating a Bot](https://core.telegram.org/bots#how-do-i-create-a-bot)
* Create a telegram bot:
  * By talking to [Bot father @BotFather](https://t.me/botfather)
  * Store bot token as environment variable `TELOXIDE_TOKEN=<tbd>` in file `.env`.
  * Store bot name as environment variable `TELOXIDE_BOT_NAME=<tbd>` in file `.env`.

### Prerequisites for building this project

The diesel cli, by default, requires openssl, libpq, sqlite, and mysql. 
```shell
sudo apt install -y libmariadbclient-dev libsqlite3-dev libpq-dev
# libmysqlclient-dev # <-- alternative to mariadb
```
Once these dependencies are installed, you can run cargo install diesel_cli.
```shell
cargo install diesel_cli
```

### Start bot without webhook

The bot is configured with environment variables.
They are stored in a file called `.env`.
For testing purposes it makes sense to work without the hazzle of setting up a proper deployment of the bot.
For production, use the webhook configuration! Otherwise, the configured health checks make no sense.

```
# cat .env
TELOXIDE_TOKEN=<tbd>
RUST_BACKTRACE=1
TELOXIDE_LOG_DIR=.
TELOXIDE_DATA_DIR=.
TELOXIDE_BOT_NAME=mybot
DATABASE_URL=sqlite://db.sqlite
```

### Start bot with webhook

There is no TLS configuration within the bot,
it expects TLS termination from a load balancer or webserver that passes plain http to the bot.
TLS is mandatory by telegram and the bot won't work without a public `https` URL.
```
# .env
TELOXIDE_TOKEN=<tbd>
RUST_BACKTRACE=1
TELOXIDE_PUBLIC_URL=https://mybot.example.com
TELOXIDE_BIND_PORT=8080
TELOXIDE_LOG_DIR=/var/log/telegrambot/
TELOXIDE_DATA_DIR=/var/lib/telegrambot/
TELOXIDE_BOT_NAME=mybot
DATABASE_URL=sqlite://db.sqlite
```
With the given configuration the bot will register the webhook at `https://mybot.example.com/bot`.
The actual bot address is under the path `/bot` because there is another path `/healthcheck` added for health checking purposes:
https://mybot.example.com/healthcheck, see `dispatch.rs` for details.

### Logging

The bot expects to run with the process id that owns the data and log directory.
It is assumed these directories are exclusively used by the bot.
Automatic log rotation is performed on a daily basis.
See logging/tracing configuration in `src/bot/core/util.rs` for details.

## Technical notes

### Telegram check health of webhook

Use this method to get current webhook status. Requires no parameters. 
On success, returns a WebhookInfo object. If the bot is using getUpdates, will return an object with the url field empty.

https://core.telegram.org/bots/api#getwebhookinfo

### Telegram deep linking

https://core.telegram.org/api/links
https://t.me/mybot?start=secret
https://t.me/<TELOXIDE_BOT_NAME>?start=secret

### Diesel-rs

https://diesel.rs/guides/all-about-inserts.html#the-returning-clause
