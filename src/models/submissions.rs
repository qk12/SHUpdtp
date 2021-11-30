use super::languages::*;
use crate::schema::submissions;
use chrono::NaiveDateTime;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub input: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeSettings {
    pub language_config: LanguageConfig,
    pub src: String,
    pub max_cpu_time: i32,
    pub max_memory: i32,
    pub test_case_id: Option<String>,
    pub test_case: Option<Vec<TestCase>>,
    pub spj_version: Option<String>,
    pub spj_config: Option<SpjConfig>,
    pub spj_compile_config: Option<SpjCompileConfig>,
    pub spj_src: Option<String>,
    pub output: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawJudgeResult {
    pub err: Option<String>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawJudgeResultData {
    pub cpu_time: i32,
    pub real_time: i32,
    pub memory: i32,
    pub signal: i32,
    pub exit_code: i32,
    pub error: i32,
    pub result: i32,
    pub test_case: String,
    pub output_md5: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeResultData {
    pub cpu_time: i32,
    pub real_time: i32,
    pub memory: i32,
    pub signal: i32,
    pub exit_code: i32,
    pub error: String,
    pub result: String,
    pub test_case: String,
    pub output_md5: Option<String>,
    pub output: Option<String>,
}

impl From<RawJudgeResultData> for JudgeResultData {
    fn from(raw: RawJudgeResultData) -> Self {
        Self {
            cpu_time: raw.cpu_time,
            real_time: raw.real_time,
            memory: raw.memory,
            signal: raw.signal,
            exit_code: raw.exit_code,
            error: {
                match raw.error {
                    0 => "SUCCESS".to_owned(),
                    -1 => "INVALID_CONFIG".to_owned(),
                    -2 => "CLONE_FAILED".to_owned(),
                    -3 => "PTHREAD_FAILED".to_owned(),
                    -4 => "WAIT_FAILED".to_owned(),
                    -5 => "ROOT_REQUIRED".to_owned(),
                    -6 => "LOAD_SECCOMP_FAILED".to_owned(),
                    -7 => "SETRLIMIT_FAILED".to_owned(),
                    -8 => "DUP2_FAILED".to_owned(),
                    -9 => "SETUID_FAILED".to_owned(),
                    -10 => "EXECVE_FAILED".to_owned(),
                    -11 => "SPJ_ERROR".to_owned(),
                    _ => "UNKNOWN_ERROR".to_owned(),
                }
            },
            result: {
                match raw.result {
                    -1 => "WRONG_ANSWER".to_owned(),
                    0 => "SUCCESS".to_owned(),
                    1 => "CPU_TIME_LIMIT_EXCEEDED".to_owned(),
                    2 => "REAL_TIME_LIMIT_EXCEEDED".to_owned(),
                    3 => "MEMORY_LIMIT_EXCEEDED".to_owned(),
                    4 => "RUNTIME_ERROR".to_owned(),
                    5 => "SYSTEM_ERROR".to_owned(),
                    _ => "UNKNOWN_ERROR".to_owned(),
                }
            },
            test_case: raw.test_case,
            output_md5: raw.output_md5,
            output: raw.output,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeResult {
    pub err: Option<String>,
    pub err_reason: Option<String>,
    pub is_accepted: Option<bool>,
    pub max_time: Option<i32>,
    pub max_memory: Option<i32>,
    pub details: Option<Vec<JudgeResultData>>,
}

impl From<RawJudgeResult> for JudgeResult {
    fn from(raw: RawJudgeResult) -> Self {
        let mut is_accepted = true;
        let raw_details: Option<Vec<RawJudgeResultData>> = if raw.err.is_none() {
            Some(serde_json::from_value::<Vec<RawJudgeResultData>>(raw.data.clone()).unwrap())
        } else {
            None
        };

        let details: Option<Vec<JudgeResultData>> = if raw_details.is_some() {
            let mut tmp = Vec::new();
            for raw_detail in raw_details.unwrap() {
                if raw_detail.result != 0 {
                    is_accepted = false;
                }
                tmp.push(JudgeResultData::from(raw_detail))
            }
            Some(tmp)
        } else {
            None
        };

        let max_time: Option<i32> = if let Some(inner_data) = details.clone() {
            Some(
                inner_data
                    .clone()
                    .iter()
                    .map(|detail| detail.cpu_time)
                    .max()
                    .unwrap_or(0),
            )
        } else {
            None
        };

        let max_memory: Option<i32> = if let Some(inner_data) = details.clone() {
            Some(
                inner_data
                    .clone()
                    .iter()
                    .map(|detail| detail.memory)
                    .max()
                    .unwrap_or(0),
            )
        } else {
            None
        };

        Self {
            err: raw.err.clone(),
            err_reason: if raw.err.is_some() {
                Some(serde_json::from_value::<String>(raw.data.clone()).unwrap())
            } else {
                None
            },
            is_accepted: if raw.err.is_none() {
                Some(is_accepted)
            } else {
                None
            },
            max_time: max_time,
            max_memory: max_memory,
            details: details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Queryable)]
pub struct RawSubmission {
    pub id: Uuid,
    pub problem_id: i32,
    pub user_id: i32,
    pub region: Option<String>,
    pub state: String,
    pub settings: String,
    pub result: Option<String>,
    pub submit_time: NaiveDateTime,
    pub is_accepted: Option<bool>,
    pub finish_time: Option<NaiveDateTime>,
    pub max_time: Option<i32>,
    pub max_memory: Option<i32>,
    pub language: Option<String>,
    pub err: Option<String>,
    pub out_results: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Insertable, Queryable)]
#[table_name = "submissions"]
pub struct InsertableSubmission {
    pub id: Uuid,
    pub problem_id: i32,
    pub user_id: i32,
    pub region: Option<String>,
    pub state: String,
    pub settings: String,
    pub result: Option<String>,
    pub submit_time: NaiveDateTime,
    pub is_accepted: Option<bool>,
    pub finish_time: Option<NaiveDateTime>,
    pub max_time: Option<i32>,
    pub max_memory: Option<i32>,
    pub language: Option<String>,
    pub err: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub id: Uuid,
    pub problem_id: i32,
    pub user_id: i32,
    pub region: Option<String>,
    pub state: String,
    pub settings: JudgeSettings,
    pub result: Option<JudgeResult>,
    pub submit_time: NaiveDateTime,
    pub is_accepted: Option<bool>,
    pub finish_time: Option<NaiveDateTime>,
    pub max_time: Option<i32>,
    pub max_memory: Option<i32>,
    pub language: Option<String>,
    pub err: Option<String>,
    pub out_results: Option<HashSet<String>>,
}

impl From<RawSubmission> for Submission {
    fn from(raw: RawSubmission) -> Self {
        Self {
            id: raw.id,
            problem_id: raw.problem_id,
            user_id: raw.user_id,
            region: raw.region,
            state: raw.state,
            settings: serde_json::from_str::<JudgeSettings>(&raw.settings).unwrap(),
            result: if let Some(result) = raw.result.clone() {
                Some(serde_json::from_str::<JudgeResult>(&result).unwrap())
            } else {
                None
            },
            submit_time: raw.submit_time,
            is_accepted: raw.is_accepted,
            finish_time: raw.finish_time,
            max_time: raw.max_time,
            max_memory: raw.max_memory,
            language: raw.language,
            err: raw.err,
            out_results: {
                if let Some(result) = raw.result {
                    let result = serde_json::from_str::<JudgeResult>(&result).unwrap();
                    if let Some(details) = result.details {
                        let mut set = std::collections::HashSet::new();
                        for detail in details {
                            set.insert(detail.result);
                        }
                        Some(set)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlimSubmission {
    pub id: Uuid,
    pub problem_id: i32,
    pub user_id: i32,
    pub username: String,
    pub state: String,
    pub submit_time: NaiveDateTime,
    pub is_accepted: Option<bool>,
    pub out_results: Option<HashSet<String>>,
    pub max_time: Option<i32>,
    pub max_memory: Option<i32>,
    pub language: Option<String>,
    pub err: Option<String>,
}

impl From<RawSubmission> for SlimSubmission {
    fn from(raw: RawSubmission) -> Self {
        Self {
            id: raw.id,
            problem_id: raw.problem_id,
            user_id: raw.user_id,
            username: "".to_string(),
            state: raw.state,
            submit_time: raw.submit_time,
            is_accepted: raw.is_accepted,
            out_results: {
                if let Some(result) = raw.result {
                    let result = serde_json::from_str::<JudgeResult>(&result).unwrap();
                    if let Some(details) = result.details {
                        let mut set = std::collections::HashSet::new();
                        for detail in details {
                            set.insert(detail.result);
                        }
                        Some(set)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            max_time: raw.max_time,
            max_memory: raw.max_memory,
            language: raw.language,
            err: raw.err,
        }
    }
}
