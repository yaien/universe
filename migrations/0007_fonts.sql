
-- fonts definition
CREATE TABLE fonts (id integer PRIMARY KEY AUTOINCREMENT,
    family varchar,
    subsets jsonb,
    variants jsonb,
    files jsonb,
    created_at datetime,
    updated_at datetime,
    provider varchar,
    category varchar,
    version varchar
);

CREATE INDEX idx_fonts_family ON fonts(family);
