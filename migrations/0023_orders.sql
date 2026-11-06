create table orders (
    id integer primary key autoincrement,
    uuid varchar not null,
    organization_id integer not null,
    cart_id integer,
    form_submission_id integer,
    user_id integer not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    amount double not null,
    provider varchar not null,
    metadata jsonb not null,
    status varchar not null,

    foreign key(cart_id) references carts(id) on delete set null,
    foreign key (form_submission_id) references form_submissions(id) on delete set null,
    foreign key (user_id) references users(id) on delete cascade,
    foreign key (organization_id) references organizations(id) on delete cascade
);

create table order_items (
    id integer primary key autoincrement,
    order_id integer not null,
    product_id integer integer not null,
    presentation_id integer not null,
    quantity integer not null,
    price_per_unit double not null,
    total_price double not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,

    foreign key (order_id) references orders(id) on delete cascade,
    foreign key (product_id) references products(id) on delete cascade,
    foreign key (presentation_id) references presentations(id) on delete cascade
);
