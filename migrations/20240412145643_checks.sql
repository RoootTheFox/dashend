-- Add migration script here
CREATE TABLE user_misc (
    id INT PRIMARY KEY NOT NULL,
    check_timeout BIGINT
);