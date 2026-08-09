create table colors (
    id integer primary key autoincrement,
    sitemap_id integer not null,
    name varchar not null,
    value varchar not null,

    foreign key(sitemap_id) references sitemaps(id) on delete cascade,
    unique (sitemap_id, name)
);

create index idx_colors on colors(sitemap_id);
