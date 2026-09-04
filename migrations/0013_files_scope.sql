alter table files add column scope varchar;

create index files_scope_idx on files(scope);

update files set scope = 'pages';
