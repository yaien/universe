pragma foreign_keys = off;

alter table actions rename to actions_old;

CREATE TABLE IF NOT EXISTS actions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sitemap_id INTEGER,
    created_at VARCHAR NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at VARCHAR NOT NULL DEFAULT CURRENT_TIMESTAMP,
    name VARCHAR NOT NULL,
    codename VARCHAR NOT NULL,
    tool VARCHAR NOT NULL,
    response_body_template VARCHAR NOT NULL DEFAULT '',
    form_id INTEGER,
    FOREIGN KEY (sitemap_id) REFERENCES sitemaps(id) ON DELETE CASCADE,
    FOREIGN KEY (form_id) REFERENCES forms(id) ON DELETE SET NULL,
    UNIQUE (sitemap_id, codename)
);

INSERT INTO actions (sitemap_id, created_at, updated_at, name, codename, tool, response_body_template)
    SELECT sitemap_id, created_at, updated_at, name, codename, tool, response_body_template
    FROM actions_old;

drop table actions_old;

pragma foreign_keys = on;
