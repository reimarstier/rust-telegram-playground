# dockerfile for reduced build time on compatible host system

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 tzdata ca-certificates

# does not compile the app itself, simply adds the release binary from the host
COPY ./target/release/rust-telegram-alias-bot /rust-telegram-alias-bot

# drop root privileges
RUN groupadd --gid 1000 telegrambot
RUN useradd --create-home --uid 1000 --gid telegrambot --shell /bin/bash telegrambot
RUN mkdir -p /var/log/telegrambot/ && chown telegrambot.telegrambot /var/log/telegrambot/
USER telegrambot

# telegram bot bind port
ENV TELOXIDE_BIND_PORT=8080
EXPOSE 8080

CMD [ "/rust-telegram-alias-bot", "bot"]
HEALTHCHECK CMD [ "/rust-telegram-alias-bot", "healthcheck"]
