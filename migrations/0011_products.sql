create table products (
    id integer primary key autoincrement,
    organization_id integer not null,
    slug varchar not null,
    name varchar not null,
    published boolean not null default false,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    deleted_at timestamp,

    foreign key (organization_id) references organizations(id) on delete cascade,
    unique(organization_id, name)
);

create index idx_products_organization_id on products(organization_id);
create index idx_products_slug on products(slug);


create table presentations (
    id integer primary key autoincrement,
    product_id integer not null,
    name varchar not null,
    quantity integer not null default 0,
    price double not null default 0.0,
    number integer not null default 0,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,

    foreign key (product_id) references products(id) on delete cascade
);

create index idx_presentations_product_id on presentations(product_id);


create table contents (
    id integer primary key autoincrement,
    presentation_id integer not null,
    file_id integer not null,
    number integer not null default 0,

    foreign key (presentation_id) references presentations(id) on delete cascade,
    foreign key (file_id) references files(id) on delete cascade
);

create index idx_contents_presentation_id on contents(presentation_id);
