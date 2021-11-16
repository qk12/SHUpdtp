use crate::schema::*;
use crate::statics::PROBLEM_TAG_NAME_CACHE;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct RawProblem {
    pub id: i32,
    pub title: String,
    pub tags: Vec<i32>,
    pub difficulty: f64,
    pub contents: String,
    pub settings: String,
    pub is_released: bool,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "problems"]
pub struct InsertableProblem {
    pub title: String,
    pub tags: Vec<i32>,
    pub difficulty: f64,
    pub contents: String,
    pub settings: String,
    pub is_released: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemInfo {
    pub title: String,
    pub tags: Vec<String>,
    pub difficulty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemContents {
    pub description: Option<String>,
    pub example_count: i32,
    pub examples: Vec<Example>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemSettings {
    pub is_spj: bool,
    pub high_performance_max_cpu_time: i32,
    pub high_performance_max_memory: i32,
    pub other_max_cpu_time: i32,
    pub other_max_memory: i32,
    pub opaque_output: bool,
    pub test_case_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct Problem {
    pub id: i32,
    pub info: ProblemInfo,
    pub contents: ProblemContents,
    pub settings: ProblemSettings,
    pub is_released: bool,
}

impl From<RawProblem> for Problem {
    fn from(raw: RawProblem) -> Self {
        let mut tag_names = Vec::new();
        {
            let lock = PROBLEM_TAG_NAME_CACHE.read().unwrap();
            for tag_id in &raw.tags {
                if let Some(tag_name) = lock.get(tag_id) {
                    tag_names.push(tag_name.clone());
                }
            }
        }

        Self {
            id: raw.id,
            info: ProblemInfo {
                title: raw.title,
                tags: tag_names,
                difficulty: raw.difficulty,
            },
            contents: serde_json::from_str::<ProblemContents>(&raw.contents).unwrap(),
            settings: serde_json::from_str::<ProblemSettings>(&raw.settings).unwrap(),
            is_released: raw.is_released,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct SlimProblem {
    pub id: i32,
    pub info: ProblemInfo,
    pub is_released: bool,
    pub is_effective: bool,
}

impl From<RawProblem> for SlimProblem {
    fn from(raw: RawProblem) -> Self {
        let mut tag_names = Vec::new();
        {
            let lock = PROBLEM_TAG_NAME_CACHE.read().unwrap();
            for tag_id in &raw.tags {
                if let Some(tag_name) = lock.get(tag_id) {
                    tag_names.push(tag_name.clone());
                }
            }
        }

        Self {
            id: raw.id,
            info: ProblemInfo {
                title: raw.title,
                tags: tag_names,
                difficulty: raw.difficulty,
            },
            is_released: raw.is_released,
            is_effective: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct OutProblem {
    pub id: i32,
    pub info: ProblemInfo,
    pub is_released: bool,
}

impl From<RawProblem> for OutProblem {
    fn from(raw: RawProblem) -> Self {
        let mut tag_names = Vec::new();
        {
            let lock = PROBLEM_TAG_NAME_CACHE.read().unwrap();
            for tag_id in &raw.tags {
                if let Some(tag_name) = lock.get(tag_id) {
                    tag_names.push(tag_name.clone());
                }
            }
        }

        Self {
            id: raw.id,
            info: ProblemInfo {
                title: raw.title,
                tags: tag_names,
                difficulty: raw.difficulty,
            },
            is_released: raw.is_released,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProblemsResult {
    pub title: String,
    pub is_success: bool,
    pub id: Option<i32>,
}

#[derive(AsChangeset)]
#[table_name = "problems"]
pub struct ProblemForm {
    pub title: Option<String>,
    pub tags: Option<Vec<i32>>,
    pub difficulty: Option<f64>,
    pub contents: Option<String>,
    pub settings: Option<String>,
}
