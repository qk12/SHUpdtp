table! {
    access_control_list (region, user_id) {
        user_id -> Int4,
        region -> Text,
        is_unrated -> Nullable<Bool>,
        is_manager -> Bool,
    }
}

table! {
    announcements (id) {
        id -> Int4,
        title -> Text,
        author -> Text,
        contents -> Text,
        release_time -> Nullable<Timestamp>,
        last_update_time -> Timestamp,
    }
}

table! {
    contests (region) {
        region -> Text,
        title -> Text,
        introduction -> Nullable<Text>,
        start_time -> Timestamp,
        end_time -> Nullable<Timestamp>,
        seal_time -> Nullable<Timestamp>,
        settings -> Text,
    }
}

table! {
    group_links (group_id, user_id) {
        group_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    groups (id) {
        id -> Int4,
        title -> Text,
        introduction -> Nullable<Text>,
    }
}

table! {
    problem_sets (region) {
        region -> Text,
        title -> Text,
        introduction -> Nullable<Text>,
    }
}

table! {
    problem_tags (id) {
        id -> Int4,
        name -> Text,
    }
}

table! {
    problems (id) {
        id -> Int4,
        title -> Text,
        tags -> Array<Int4>,
        difficulty -> Float8,
        contents -> Text,
        settings -> Text,
        is_released -> Bool,
    }
}

table! {
    region_access_settings (region) {
        region -> Text,
        salt -> Nullable<Varchar>,
        hash -> Nullable<Bytea>,
    }
}

table! {
    region_links (inner_id, region) {
        region -> Text,
        inner_id -> Int4,
        problem_id -> Int4,
        score -> Nullable<Int4>,
    }
}

table! {
    regions (name, self_type) {
        name -> Text,
        self_type -> Text,
        title -> Text,
        has_access_setting -> Bool,
        introduction -> Nullable<Text>,
    }
}

table! {
    samples (submission_id) {
        submission_id -> Uuid,
        description -> Nullable<Text>,
    }
}

table! {
    submissions (id) {
        id -> Uuid,
        problem_id -> Int4,
        user_id -> Int4,
        region -> Nullable<Text>,
        state -> Text,
        settings -> Text,
        result -> Nullable<Text>,
        submit_time -> Timestamp,
        is_accepted -> Nullable<Bool>,
        finish_time -> Nullable<Timestamp>,
        max_time -> Nullable<Int4>,
        max_memory -> Nullable<Int4>,
        language -> Nullable<Text>,
        err -> Nullable<Text>,
        out_results -> Nullable<Array<Text>>,
    }
}

table! {
    users (id) {
        id -> Int4,
        salt -> Nullable<Varchar>,
        hash -> Nullable<Bytea>,
        username -> Text,
        email -> Text,
        role -> Text,
        real_name -> Nullable<Text>,
        school -> Nullable<Text>,
        student_number -> Nullable<Text>,
        profile_picture -> Text,
        reset_password_token_hash -> Nullable<Bytea>,
        reset_password_token_expiration_time -> Nullable<Timestamp>,
    }
}

allow_tables_to_appear_in_same_query!(
    access_control_list,
    announcements,
    contests,
    group_links,
    groups,
    problem_sets,
    problem_tags,
    problems,
    region_access_settings,
    region_links,
    regions,
    samples,
    submissions,
    users,
);
