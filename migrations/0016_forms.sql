create table forms (
    id integer primary key autoincrement,
    organization_id integer not null,
    codename varchar not null,
    name varchar not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    deleted_at timestamp,

    foreign key (organization_id) references organizations(id),
    unique (organization_id, codename)
);

create table form_fields (
    id integer primary key autoincrement,
    form_id integer not null,
    name varchar not null,
    label varchar not null,
    number integer not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    deleted_at timestamp,

    foreign key (form_id) references forms(id) on delete cascade,
    unique (form_id, name)
);

create table form_submissions (
    id integer primary key autoincrement,
    form_id integer not null,
    user_id integer,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    deleted_at timestamp,

    foreign key (form_id) references forms(id) on delete cascade
);

create index idx_form_submission_form_id on form_submissions(form_id);

create table form_submission_answers (
    id integer primary key autoincrement,
    submission_id integer not null,
    field_id integer not null,
    value varchar not null,
    created_at timestamp not null default current_timestamp,
    updated_at timestamp not null default current_timestamp,
    deleted_at timestamp,

    foreign key (submission_id) references form_submissions(id) on delete cascade,
    foreign key (field_id) references form_fields(id) on delete cascade
);


create index idx_form_submission_answers_submission_id on form_submission_answers(submission_id);
create index idx_form_submission_answers_field_id_value on form_submission_answers(field_id, value);
