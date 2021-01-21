table! {
    sql_allocation_utxo (id) {
        id -> Integer,
        sql_asset_id -> Integer,
        txid -> Text,
        vout -> Integer,
    }
}

table! {
    sql_allocations (id) {
        id -> Integer,
        sql_allocation_utxo_id -> Integer,
        node_id -> Text,
        assignment_index -> Integer,
        amount -> BigInt,
        blinding -> Text,
    }
}

table! {
    sql_assets (id) {
        id -> Integer,
        contract_id -> Text,
        ticker -> Text,
        asset_name -> Text,
        asset_description -> Nullable<Text>,
        known_circulating_supply -> BigInt,
        is_issued_known -> Nullable<Bool>,
        max_cap -> BigInt,
        chain -> Text,
        fractional_bits -> Binary,
        asset_date -> Timestamp,
    }
}

table! {
    sql_inflation (id) {
        id -> Integer,
        sql_asset_id -> Integer,
        outpoint_txid -> Nullable<Text>,
        outpoint_vout -> Nullable<Integer>,
        accounting_amount -> BigInt,
    }
}

table! {
    sql_issues (id) {
        id -> Integer,
        sql_asset_id -> Integer,
        node_id -> Text,
        contract_id -> Text,
        amount -> BigInt,
        origin_txid -> Nullable<Text>,
        origin_vout -> Nullable<Integer>,
    }
}

allow_tables_to_appear_in_same_query!(
    sql_allocation_utxo,
    sql_allocations,
    sql_assets,
    sql_inflation,
    sql_issues,
);
