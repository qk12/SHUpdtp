use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "group_links"]
pub struct GroupLink {
    pub group_id: i32,
    pub user_id: i32,
}
