use diesel::prelude::*;
use server_core::errors::*;

pub fn is_in_group(conn: &PgConnection, group_id: i32, user_id: i32) -> ServiceResult<bool> {
    use crate::schema::group_links as group_links_schema;

    if group_links_schema::table
        .filter(group_links_schema::group_id.eq(group_id))
        .filter(group_links_schema::user_id.eq(user_id))
        .count()
        .get_result::<i64>(conn)?
        == 1
    {
        Ok(true)
    } else {
        Ok(false)
    }
}
