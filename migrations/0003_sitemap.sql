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

CREATE INDEX IF NOT EXISTS idx_sitemap_lookup ON sitemaps(organization_id, branch);

CREATE TABLE IF NOT EXISTS pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sitemap_id INTEGER NOT NULL,
    layout_id INTEGER,
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    title TEXT NOT NULL,
    og_image_file_id INTEGER,
    og_description TEXT,
    og_type TEXT,
    html TEXT NOT NULL DEFAULT '',
    css TEXT NOT NULL DEFAULT '',
    js TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (sitemap_id) REFERENCES sitemaps(id) ON DELETE CASCADE,
    FOREIGN KEY (layout_id) REFERENCES layouts(id) ON DELETE SET NULL,
    UNIQUE (sitemap_id, path),
    UNIQUE (sitemap_id, name)
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
    UNIQUE (sitemap_id, name)
);


CREATE INDEX IF NOT EXISTS idx_emails_lookup ON emails(sitemap_id, name);
