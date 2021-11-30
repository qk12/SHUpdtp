use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Queryable)]
#[table_name = "regions"]
pub struct Region {
    pub name: String,
    pub can_view_testcases: bool,
}

#[derive(AsChangeset)]
#[table_name = "regions"]
pub struct RegionForm {
    pub can_view_testcases: Option<bool>,
}
