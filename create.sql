CREATE TABLE IF NOT EXISTS "uris" (
    "id" TEXT PRIMARY KEY,
    "url" TEXT NOT NULL,
    "scheme" TEXT NOT NULL,
    "host" TEXT,
    "path" TEXT,
    "live_status" TEXT,
    "title" TEXT,
    "auto_descr" TEXT,
    "man_descr" TEXT,
    "crea_user" TEXT DEFAULT 'system',
    "crea_time" TEXT,
    "modi_user" TEXT DEFAULT 'system',
    "modi_time" TEXT
);