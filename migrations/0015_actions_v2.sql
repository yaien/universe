DROP TABLE IF EXISTS actions;
DROP INDEX IF EXISTS idx_actions_sitemap_id;
DROP INDEX IF EXISTS idx_actions_name;

CREATE TABLE IF NOT EXISTS actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sitemap_id INTEGER,
    created_at VARCHAR NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at VARCHAR NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name VARCHAR NOT NULL,
    codename VARCHAR NOT NULL,
    tool VARCHAR NOT NULL,
    response_body_template VARCHAR NOT NULL DEFAULT '',
    FOREIGN KEY (sitemap_id) REFERENCES sitemaps(id) ON DELETE CASCADE,
    UNIQUE (sitemap_id, codename)
);
