table! {
    folders (id) {
        id -> Nullable<Text>,
        title -> Text,
        parent_id -> Text,
    }
}

table! {
    notes (id) {
        id -> Nullable<Text>,
        parent_id -> Text,
        title -> Text,
        body -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    folders,
    notes,
);
