-- initial authentication :3
CREATE TABLE users (
    id INT PRIMARY KEY NOT NULL,
    token TEXT,
    token_expiration BIGINT
);

-- initial profile table
CREATE TABLE profiles (
    id INT PRIMARY KEY NOT NULL,
    bio TEXT, -- nudes in bio
    pronouns TINYTEXT, -- los pronombres hermanos --
    --- social links ---
    website           TINYTEXT, -- yeah
    social_github     TINYTEXT, -- @username
    social_bluesky    TINYTEXT, -- @domain.tld or @user.bsky.social
    social_fediverse  TINYTEXT, -- @user@host.tld
    social_discord    TINYTEXT, -- username:snowflake
    social_matrix     TINYTEXT, -- MXID (user:host.tld)
    social_tumblr     TINYTEXT, -- idfk
    social_myspace    TINYTEXT, -- yeah idk either
    social_facebook   TINYTEXT  -- idk just a username ig?
);
