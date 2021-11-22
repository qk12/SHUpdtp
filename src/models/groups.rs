use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct Group {
    pub id: i32,
    pub title: String,
    pub introduction: Option<String>,
}

#[derive(Debug, Insertable)]
#[table_name = "groups"]
pub struct InsertableGroup {
    pub title: String,
    pub introduction: Option<String>,
}

#[derive(AsChangeset)]
#[table_name = "groups"]
pub struct GroupForm {
    pub title: Option<String>,
    pub introduction: Option<String>,
}

#[derive(Serialize)]
pub struct OutGroup {
    pub id: i32,
    pub title: String,
}

impl From<Group> for OutGroup {
    fn from(group: Group) -> Self {
        Self {
            id: group.id,
            title: group.title,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertUserIntoGroupResult {
    pub user_id: i32,
    pub is_success: bool,
}
