table! {
    signatures (id) {
        id -> Int4,
        signature -> Nullable<Varchar>,
        slot -> Nullable<Int4>,
        err -> Nullable<Text>,
        memo -> Nullable<Text>,
        block_time -> Nullable<Int4>,
        confirmation_status -> Nullable<Varchar>,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        signature -> Nullable<Varchar>,
        slot -> Nullable<Int4>,
        transaction -> Nullable<Text>,
        block_time -> Nullable<Int4>,
    }
}

allow_tables_to_appear_in_same_query!(
    signatures,
    transactions,
);
