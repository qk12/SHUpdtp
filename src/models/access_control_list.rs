use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[table_name = "access_control_list"]
pub struct AccessControlListColumn {
    pub self_type: String,
    pub id: i32,
    pub region: String,
    pub is_unrated: Option<bool>,
    pub is_manager: bool,
}
