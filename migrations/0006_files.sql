create table files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    organization_id INTEGER NOT NULL,
    name STRING,
    preset STRING,
    created_at DATETIME default CURRENT_TIMESTAMP,
    updated_at DATETIME default CURRENT_TIMESTAMP,

    FOREIGN KEY(organization_id) REFERENCES organizations(id) ON DELETE CASCADE,
    UNIQUE (organization_id, name)
);

CREATE INDEX idx_files_lookup ON files(name, organization_id);

CREATE TABLE files_formats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id INTEGER NOT NULL,
    variant INTEGER NOT NULL,
    size INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    content_type VARCHAR NOT NULL,

    FOREIGN KEY(file_id) REFERENCES files(id) ON DELETE CASCADE;
    UNIQUE(file_id, variant)
);

CREATE INDEX idx_files_format_lookup ON files_formats(file_id, variant)
