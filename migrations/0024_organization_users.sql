create table organization_users (
    id integer primary key autoincrement,
    organization_id integer not null,
    user_id integer not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,

    unique (organization_id, user_id),
    foreign key (organization_id) references organizations(id) on delete cascade,
    foreign key (user_id) references users(id) on delete cascade
);
