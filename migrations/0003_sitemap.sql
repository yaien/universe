CREATE TABLE IF NOT EXISTS sitemaps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    organization_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    UNIQUE (organization_id, name)
);

CREATE INDEX IF NOT EXISTS idx_sitemap_lookup ON sitemaps(organization_id, name);

CREATE TABLE IF NOT EXISTS pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sitemap_id INTEGER NOT NULL,
    layout_id INTEGER,
    path TEXT NOT NULL,
    html TEXT NOT NULL DEFAULT '',
    css TEXT NOT NULL DEFAULT '',
    js TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (sitemap_id) REFERENCES sitemaps(id) ON DELETE CASCADE,
    UNIQUE (sitemap_id, path)
);

CREATE INDEX IF NOT EXISTS idx_pages_lookup ON pages(sitemap_id, path);

CREATE TABLE IF NOT EXISTS layouts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sitemap_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    html TEXT NOT NULL DEFAULT '',
    css TEXT NOT NULL DEFAULT '',
    js TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (sitemap_id) REFERENCES sitemaps(id) ON DELETE CASCADE,
    UNIQUE (sitemap_id, name)
);

CREATE INDEX IF NOT EXISTS idx_layout_lookup ON layouts(sitemap_id, name);

CREATE TABLE IF NOT EXISTS emails (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sitemap_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    subject TEXT NOT NULL DEFAULT '',
    body TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (sitemap_id) REFERENCES sitemaps(id) ON DELETE CASCADE,
    UNIQUE (sitemap_id, subject)
);


CREATE INDEX IF NOT EXISTS idx_emails_lookup ON emails(sitemap_id, name);
