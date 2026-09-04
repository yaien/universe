create index idx_products_deleted_at on products (deleted_at);

alter table presentations add column deleted_at timestamp;

create index idx_presentations_deleted_at on presentations (deleted_at);
