CREATE TABLE IF NOT EXISTS vars (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR,
    source VARCHAR,
    query VARCHAR,
    args JSONB,
    test_args JSONB,
    page_id INTEGER,
    layout_id INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (page_id) REFERENCES pages (id) ON DELETE CASCADE,
    FOREIGN KEY (layout_id) REFERENCES layouts (id) ON DELETE CASCADE,
);

CREATE INDEX IF NOT EXISTS idx_vars_page_id ON vars (page_id);
CREATE INDEX IF NOT EXISTS idx_vars_layout_id ON vars (layout_id);
