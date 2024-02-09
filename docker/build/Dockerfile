FROM rust:1.76-bookworm as builder

WORKDIR /app

# add source code
COPY ../.. .

# install rust app from path to /usr/local/cargo/bin/
RUN cargo install --path /app

# display generate binaries with find command
RUN find /usr/local/cargo/bin/

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 tzdata ca-certificates

COPY --from=builder /usr/local/cargo/bin/rust-telegram-alias-bot /usr/local/cargo/bin/rust-telegram-alias-bot

# drop root privileges
RUN groupadd --gid 1000 telegrambot
RUN useradd --create-home --uid 1000 --gid telegrambot --shell /bin/bash telegrambot
RUN mkdir -p /var/log/telegrambot/ && chown telegrambot.telegrambot /var/log/telegrambot/
USER telegrambot

# telegram bot bind port
ENV TELOXIDE_BIND_PORT=8080
EXPOSE 8080

CMD [ "/usr/local/cargo/bin/rust-telegram-alias-bot", "bot"]
HEALTHCHECK CMD [ "/usr/local/cargo/bin/rust-telegram-alias-bot", "healthcheck"]
