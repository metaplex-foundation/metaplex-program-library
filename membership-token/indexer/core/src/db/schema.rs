table! {
    loading_statuses (id) {
        id -> Int4,
        description -> Nullable<Varchar>,
    }
}

table! {
    signatures (id) {
        id -> Int4,
        signature -> Nullable<Varchar>,
        slot -> Nullable<Int4>,
        err -> Nullable<Text>,
        memo -> Nullable<Text>,
        block_time -> Nullable<Int4>,
        confirmation_status -> Nullable<Varchar>,
        loading_status -> Nullable<Int4>,
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

joinable!(signatures -> loading_statuses (loading_status));

allow_tables_to_appear_in_same_query!(
    loading_statuses,
    signatures,
    transactions,
);
