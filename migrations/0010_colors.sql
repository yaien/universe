create table colors (
    id integer primary key autoincrement,
    sitemap_id integer not null,
    tag varchar not null,
    value varchar not null,
    created_at datetime not null default current_timestamp,
    updated_at datetime not null default current_timestamp,

    foreign key(sitemap_id) references sitemaps(id) on delete cascade,
    unique (sitemap_id, tag)
);

create index idx_colors on colors(sitemap_id);
