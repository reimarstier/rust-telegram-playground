-- Your SQL goes here
CREATE TABLE `users`(
    `id` INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    `name` VARCHAR NOT NULL,
    `start` VARCHAR NOT NULL,
    `role` VARCHAR NOT NULL,
    UNIQUE(name),
    UNIQUE(start)
);

CREATE TABLE `telegram_accounts`(
  `id` BIGINT NOT NULL PRIMARY KEY,
  `user_id` INTEGER NOT NULL,
  FOREIGN KEY (user_id) REFERENCES users (id)
);

