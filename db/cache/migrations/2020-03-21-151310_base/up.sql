-- Your SQL goes here

create table sql_assets(
    id INTEGER PRIMARY KEY not null,
    contract_id text not null,
    ticker text not null,
    asset_name text not null,
    asset_description text,
    known_circulating_supply bigint not null,
    is_issued_known boolean,
    max_cap bigint not null,
    chain text not null,
    fractional_bits blob not null,
    asset_date datetime not null
);

SELECT diesel_manage_updated_at('sql_assets');

create table sql_issues(
    id integer PRIMARY key not null,
    sql_asset_id integer not null,
    node_id text not null,
    contract_id text not null,
    amount bigint not null,
    origin_txid text,
    origin_vout integer
);

create table sql_inflation(
    id integer PRIMARY KEY not null,
    sql_asset_id integer not null,
    outpoint_txid text,
    outpoint_vout integer,
    accounting_amount bigint not null
); 

create table sql_allocation_utxo(
    id INTEGER PRIMARY key not null,
    sql_asset_id integer not null,
    txid text not null,
    vout INTEGER not null
);

create table sql_allocations(
    id INTEGER PRIMARY key not null,
    sql_allocation_utxo_id integer not null,
    node_id text not null,
    assignment_index integer not null,
    amount bigint not null,
    blinding text not null 
);