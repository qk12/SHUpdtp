use crate::schema::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct RawProblem {
    pub id: i32,
    pub title: String,
    pub tags: Vec<String>,
    pub difficulty: f64,
    pub contents: String,
    pub settings: String,
    pub is_released: bool,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "problems"]
pub struct InsertableProblem {
    pub title: String,
    pub tags: Vec<String>,
    pub difficulty: f64,
    pub contents: String,
    pub settings: String,
    pub is_released: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, juniper::GraphQLObject)]
pub struct Info {
    pub title: String,
    pub tags: Vec<String>,
    pub difficulty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, juniper::GraphQLObject)]
pub struct Example {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, juniper::GraphQLObject)]
pub struct Contents {
    pub description: Option<String>,
    pub example_count: i32,
    pub examples: Vec<Example>,
}

#[derive(Debug, Clone, Serialize, Deserialize, juniper::GraphQLObject)]
pub struct Settings {
    pub is_spj: bool,
    pub high_performance_max_cpu_time: i32,
    pub high_performance_max_memory: i32,
    pub other_max_cpu_time: i32,
    pub other_max_memory: i32,
    pub opaque_output: bool,
    pub test_case_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, juniper::GraphQLObject)]
pub struct Problem {
    pub id: i32,
    pub info: Info,
    pub contents: Contents,
    pub settings: Settings,
    pub is_released: bool,
}

impl From<RawProblem> for Problem {
    fn from(raw: RawProblem) -> Self {
        Self {
            id: raw.id,
            info: Info {
                title: raw.title,
                tags: raw.tags,
                difficulty: raw.difficulty,
            },
            contents: serde_json::from_str::<Contents>(&raw.contents).unwrap(),
            settings: serde_json::from_str::<Settings>(&raw.settings).unwrap(),
            is_released: raw.is_released,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, juniper::GraphQLObject)]
pub struct SlimProblem {
    pub id: i32,
    pub info: Info,
    pub is_released: bool,
}

impl From<RawProblem> for SlimProblem {
    fn from(raw: RawProblem) -> Self {
        Self {
            id: raw.id,
            info: Info {
                title: raw.title,
                tags: raw.tags,
                difficulty: raw.difficulty,
            },
            is_released: raw.is_released,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, juniper::GraphQLObject)]
pub struct CreateProblemsResult {
    pub title: String,
    pub is_success: bool,
    pub id: Option<i32>,
}