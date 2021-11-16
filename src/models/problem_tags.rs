use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct ProblemTag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Insertable)]
#[table_name = "problem_tags"]
pub struct InsertableProblemTag {
    pub name: String,
}

#[derive(AsChangeset)]
#[table_name = "problem_tags"]
pub struct ProblemTagForm {
    pub name: Option<String>,
}
