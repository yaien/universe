create table sitemaps_fonts (
    id integer primary key autoincrement,
    sitemap_id integer not null,
    font_id integer not null,
    tag varchar not null,

    foreign key (sitemap_id) references sitemaps(id) on delete cascade,
    foreign key (font_id) references fonts(id) on delete cascade,
    unique (sitemap_id, tag)
);


create index idx_sitemaps_fonts on sitemaps_fonts(sitemap_id, font_id);
