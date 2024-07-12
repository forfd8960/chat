-- Add migration script here
CREATE TABLE IF NOT EXISTS users (
    id bigserial PRIMARY KEY,
    ws_id bigint NOT NULL,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL,
    password VARCHAR(97) NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- workspaces for users
CREATE TABLE IF NOT EXISTS workspaces (
    id bigserial PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id bigint NOT NULL REFERENCES users(id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE chat_type AS ENUM (
    'single',
    'group',
    'private_channel',
    'public_channel'
);

CREATE TABLE IF NOT EXISTS chats (
    id bigserial PRIMARY KEY,
    ws_id bigint NOT NULL REFERENCES workspaces(id),
    name VARCHAR(64) NOT NULL,
    type chat_type NOT NULL,
    members bigint[] NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS messages (
    id bigserial PRIMARY KEY,
    chat_id bigint NOT NULL REFERENCES chats(id),
    sender_id bigint NOT NULL REFERENCES users(id),
    content text NOT NULL,
    files text[] DEFAULT '{}',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- Create an index on chat_id and created_at
CREATE INDEX IF NOT EXISTS idx_messages_chat_id_created_at ON messages(chat_id, created_at DESC);

-- Create an index on chat_id and updated_at
CREATE INDEX IF NOT EXISTS idx_messages_chat_id_updated_at ON messages(chat_id, updated_at DESC);

-- Create an index on sender_id and created_at
CREATE INDEX IF NOT EXISTS idx_messages_sender_id_created_at ON messages(sender_id, created_at DESC);