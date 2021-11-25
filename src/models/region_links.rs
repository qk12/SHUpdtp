use super::problems;
use crate::schema::*;
use crate::statics::PROBLEM_TAG_NAME_CACHE;
use server_core::errors::ServiceResult;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Queryable)]
#[table_name = "region_links"]
pub struct RegionLink {
    pub region: String,
    pub inner_id: i32,
    pub problem_id: i32,
    pub score: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRegionLinksResult {
    pub problem_id: i32,
    pub inner_id: Option<i32>,
    pub is_success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct RawLinkedProblemColumn {
    pub region: String,
    pub inner_id: i32,
    pub problem_id: i32,
    pub problem_title: String,
    pub problem_tags: Vec<i32>,
    pub problem_difficulty: f64,
    pub is_released: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedProblemColumn {
    pub region: String,
    pub inner_id: i32,
    pub is_accepted: bool,
    pub out_problem: problems::OutProblem,
    pub submit_times: i32,
    pub accept_times: i32,
    pub error_times: i32,
}

use crate::models::statistics::get_results;
use server_core::database::*;
pub fn get_column_from_raw(
    conn: &PooledConnection,
    raw: RawLinkedProblemColumn,
    user_id: Option<i32>,
) -> ServiceResult<LinkedProblemColumn> {
    let statistic = get_results(conn, raw.region.clone(), raw.problem_id)?;

    let mut tag_names = Vec::new();
    {
        let lock = PROBLEM_TAG_NAME_CACHE.read().unwrap();
        for tag_id in &raw.problem_tags {
            if let Some(tag_name) = lock.get(tag_id) {
                tag_names.push(tag_name.clone());
            }
        }
    }

    Ok(LinkedProblemColumn {
        region: raw.region,
        inner_id: raw.inner_id,
        is_accepted: if let Some(inner_id) = user_id {
            statistic.accepted_user_list.contains(&inner_id)
        } else {
            false
        },
        out_problem: problems::OutProblem {
            id: raw.problem_id,
            info: problems::ProblemInfo {
                title: raw.problem_title,
                tags: tag_names,
                difficulty: raw.problem_difficulty,
            },
            is_released: raw.is_released,
        },
        submit_times: statistic.submit_times,
        accept_times: statistic.accept_times,
        error_times: statistic.error_times,
    })
}
