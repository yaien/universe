pragma foreign_keys = off;

alter table forms rename to forms_old;

create table forms (
    id integer primary key autoincrement,
    organization_id integer not null,
    name varchar not null,
    requires_login boolean not null default false,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    deleted_at timestamp,

    foreign key (organization_id) references organizations(id),
    unique (organization_id, name)
);


insert into forms (id, organization_id, name, created_at, updated_at)
select id, organization_id, name, created_at, updated_at
from forms_old;

drop table forms_old;

pragma foreign_keys = on;
