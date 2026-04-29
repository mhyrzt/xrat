ALTER TABLE runtime_sessions
ADD COLUMN socks_host TEXT;

ALTER TABLE runtime_sessions
ADD COLUMN socks_port BIGINT;

ALTER TABLE runtime_sessions
ADD COLUMN http_host TEXT;

ALTER TABLE runtime_sessions
ADD COLUMN http_port BIGINT;

ALTER TABLE runtime_sessions
ADD COLUMN shadowsocks_host TEXT;

ALTER TABLE runtime_sessions
ADD COLUMN shadowsocks_port BIGINT;
