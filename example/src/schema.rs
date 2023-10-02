// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        email -> Text,
        created_at -> Timestamp,
    }
}
