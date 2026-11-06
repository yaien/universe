create table carts (
    id integer primary key autoincrement,
    organization_id integer not null,
    user_id integer,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    deleted_at timestamp,

    foreign key (user_id) references users(id) on delete cascade,
    foreign key (organization_id) references organizations(id) on delete cascade
);


create table cart_items (
    id integer primary key autoincrement,
    cart_id integer not null,
    product_id integer integer not null,
    presentation_id integer not null,
    quantity integer not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,

    foreign key (cart_id) references carts(id) on delete cascade,
    foreign key (product_id) references products(id) on delete cascade,
    foreign key (presentation_id) references presentations(id) on delete cascade
);
