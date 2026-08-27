BEGIN TRANSACTION;

ALTER TABLE sitemaps RENAME TO sitemaps_old; 

CREATE TABLE IF NOT EXISTS sitemaps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    organization_id INTEGER NOT NULL,
    branch TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    bundled_css TEXT,
    bundled_js TEXT,
    favicon_file_id INTEGER,

    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    FOREIGN KEY (favicon_file_id) REFERENCES files(id) ON DELETE SET NULL,
    UNIQUE (organization_id, branch)
);

INSERT INTO sitemaps(id, organization_id, branch, created_at, updated_at, bundled_css, bundled_js)
    SELECT (id, organization_id, branch, created_at, updated_at, bundled_css, bundled_js) from sitemaps_old;

COMMIT;
