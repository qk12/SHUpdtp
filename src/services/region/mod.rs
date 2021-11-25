pub mod utils;

use crate::judge_actor::JudgeActorAddr;
use crate::models::problems::*;
use crate::models::region_links::*;
use crate::models::regions::*;
use crate::models::utils::SizedList;
use actix_files::NamedFile;
use actix_web::web;
use diesel::prelude::*;
use server_core::database::{db_connection, Pool};
use server_core::errors::{ServiceError, ServiceResult};
use uuid::Uuid;

pub fn get_list(
    self_type: Option<String>,
    limit: i32,
    offset: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<SizedList<Region>> {
    let conn = &db_connection(&pool)?;

    use crate::schema::regions as regions_schema;
    let target = regions_schema::table.filter(
        regions_schema::self_type
            .nullable()
            .eq(self_type.clone())
            .or(self_type.is_none()),
    );

    let total: i64 = target.clone().count().get_result(conn)?;

    let regions: Vec<Region> = target
        .offset(offset.into())
        .limit(limit.into())
        .load(conn)?;

    Ok(SizedList {
        total: total,
        list: regions,
    })
}

pub fn insert_problems(
    region: String,
    problem_ids: Vec<i32>,
    score: Option<i32>,
    pool: web::Data<Pool>,
) -> ServiceResult<Vec<CreateRegionLinksResult>> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problems as problems_schema;
    use crate::schema::region_links as region_links_schema;

    let mut target_id: i32 = region_links_schema::table
        .select(region_links_schema::inner_id)
        .filter(region_links_schema::region.eq(region.clone()))
        .order(region_links_schema::inner_id.desc())
        .first(conn)
        .unwrap_or(0);
    target_id += 1;

    let mut res = Vec::new();
    for problem_id in problem_ids {
        match diesel::insert_into(region_links_schema::table)
            .values(&RegionLink {
                region: region.clone(),
                inner_id: target_id,
                problem_id: problem_id,
                score: Some(score.unwrap_or(100)),
            })
            .execute(conn)
        {
            Ok(_) => {
                diesel::update(problems_schema::table.filter(problems_schema::id.eq(problem_id)))
                    .set(problems_schema::is_released.eq(true))
                    .execute(conn)
                    .expect("Error changing problem's release state.");
                res.push(CreateRegionLinksResult {
                    problem_id: problem_id,
                    inner_id: Some(target_id),
                    is_success: true,
                });
                target_id += 1;
            }
            Err(_) => {
                res.push(CreateRegionLinksResult {
                    problem_id: problem_id,
                    inner_id: None,
                    is_success: false,
                });
            }
        }
    }

    Ok(res)
}

pub fn get_linked_problem_column_list(
    region: String,
    user_id: Option<i32>,
    inner_id_filter: Option<i32>,
    problem_id_filter: Option<i32>,
    title_filter: Option<String>,
    tag_filter: Option<Vec<i32>>,
    difficulty_filter: Option<String>,
    id_order: Option<bool>,
    problem_id_order: Option<bool>,
    difficulty_order: Option<bool>,
    limit: i32,
    offset: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<SizedList<LinkedProblemColumn>> {
    let conn = &db_connection(&pool)?;

    let title_filter = if let Some(inner_data) = title_filter {
        Some(String::from("%") + &inner_data.as_str().replace(" ", "%") + "%")
    } else {
        None
    };

    let tag_filter: Vec<i32> = if let Some(inner_data) = tag_filter {
        inner_data.clone()
    } else {
        Vec::<i32>::new()
    };

    let (min_difficulty, max_difficulty) = if difficulty_filter.is_none() {
        (0.0, 10.0)
    } else {
        match difficulty_filter.unwrap().as_str() {
            "Navie" => (0.0, 2.5),
            "Easy" => (2.5, 5.0),
            "Middle" => (5.0, 7.5),
            "Hard" => (7.5, 10.0),
            _ => (0.0, 10.0),
        }
    };

    use crate::schema::problems as problems_schema;
    use crate::schema::region_links as region_links_schema;
    let target = region_links_schema::table
        .inner_join(
            problems_schema::table.on(problems_schema::id.eq(region_links_schema::problem_id)),
        )
        .filter(region_links_schema::region.eq(region))
        .filter(
            region_links_schema::inner_id
                .nullable()
                .eq(inner_id_filter)
                .or(inner_id_filter.is_none()),
        )
        .filter(
            problems_schema::id
                .nullable()
                .eq(problem_id_filter)
                .or(problem_id_filter.is_none()),
        )
        .filter(
            problems_schema::tags
                .overlaps_with(tag_filter.clone())
                .or(tag_filter.is_empty()),
        )
        .filter(
            problems_schema::title
                .nullable()
                .like(title_filter.clone())
                .or(title_filter.is_none()),
        )
        .filter(problems_schema::difficulty.between(min_difficulty, max_difficulty));

    let total: i64 = target.clone().count().get_result(conn)?;

    let target = target.offset(offset.into()).limit(limit.into()).select((
        region_links_schema::region,
        region_links_schema::inner_id,
        problems_schema::id,
        problems_schema::title,
        problems_schema::tags,
        problems_schema::difficulty,
        problems_schema::is_released,
    ));

    let columns: Vec<RawLinkedProblemColumn> = match id_order {
        None => match problem_id_order {
            None => match difficulty_order {
                None => target.load(conn)?,
                Some(true) => target.order(problems_schema::difficulty.asc()).load(conn)?,
                Some(false) => target
                    .order(problems_schema::difficulty.desc())
                    .load(conn)?,
            },
            Some(true) => target.order(problems_schema::id.asc()).load(conn)?,
            Some(false) => target.order(problems_schema::id.desc()).load(conn)?,
        },
        Some(true) => target
            .order(region_links_schema::inner_id.asc())
            .load(conn)?,
        Some(false) => target
            .order(region_links_schema::inner_id.desc())
            .load(conn)?,
    };

    let out_columns = {
        let mut res = Vec::new();
        for column in columns {
            res.push(get_column_from_raw(conn, column, user_id)?);
        }
        res
    };

    Ok(SizedList {
        total: total,
        list: out_columns,
    })
}

pub fn get_linked_problem(
    region: String,
    inner_id: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<Problem> {
    let conn = &db_connection(&pool)?;

    use crate::schema::region_links as region_links_schema;

    let problem_id: i32 = region_links_schema::table
        .filter(region_links_schema::region.eq(region))
        .filter(region_links_schema::inner_id.eq(inner_id))
        .select(region_links_schema::problem_id)
        .first(conn)?;

    use crate::schema::problems as problems_schema;

    let problem: RawProblem = problems_schema::table
        .filter(problems_schema::id.eq(problem_id))
        .first(conn)?;

    Ok(Problem::from(problem))
}

pub fn create_submission(
    region: String,
    inner_id: i32,
    user_id: i32,
    src: String,
    language: String,
    pool: web::Data<Pool>,
    judge_actor: web::Data<JudgeActorAddr>,
) -> ServiceResult<Uuid> {
    let conn = &db_connection(&pool)?;

    use crate::schema::region_links as region_links_schema;

    let problem_id: i32 = region_links_schema::table
        .filter(region_links_schema::region.eq(region.clone()))
        .filter(region_links_schema::inner_id.eq(inner_id))
        .select(region_links_schema::problem_id)
        .first(conn)?;

    use crate::schema::problems as problems_schema;

    let is_released: bool = problems_schema::table
        .filter(problems_schema::id.eq(problem_id))
        .select(problems_schema::is_released)
        .first(conn)?;

    if !is_released {
        let hint = "Problem is not released.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    use crate::services::submission::create as inner_create;

    inner_create(
        Some(region),
        problem_id,
        user_id,
        src,
        language,
        pool,
        judge_actor,
    )
}

pub fn delete_problem(region: String, inner_id: i32, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::region_links as region_links_schema;

    let target = region_links_schema::table.filter(
        region_links_schema::region
            .eq(region.clone())
            .and(region_links_schema::inner_id.eq(inner_id.clone())),
    );

    let problem_id: i32 = target
        .clone()
        .select(region_links_schema::problem_id)
        .first(conn)?;

    diesel::delete(target).execute(conn)?;

    if region_links_schema::table
        .filter(region_links_schema::problem_id.eq(problem_id.clone()))
        .count()
        .get_result::<i64>(conn)? as i32
        == 0
    {
        use crate::schema::problems as problems_schema;
        diesel::update(problems_schema::table.filter(problems_schema::id.eq(problem_id)))
            .set(problems_schema::is_released.eq(false))
            .execute(conn)
            .expect("Error changing problem's release state.");
    }

    use crate::schema::submissions as submissions_schema;
    diesel::delete(
        submissions_schema::table.filter(
            submissions_schema::region
                .eq(region.clone())
                .and(submissions_schema::problem_id.eq(problem_id.clone())),
        ),
    )
    .execute(conn)?;

    Ok(())
}

pub fn get_linked_problem_test_case(
    region: String,
    inner_id: i32,
    test_case_id: i32,
    input: bool,
    pool: web::Data<Pool>,
) -> ServiceResult<NamedFile> {
    let conn = &db_connection(&pool)?;

    use crate::schema::region_links as region_links_schema;
    let problem_id: i32 = region_links_schema::table
        .filter(region_links_schema::region.eq(region))
        .filter(region_links_schema::inner_id.eq(inner_id))
        .select(region_links_schema::problem_id)
        .first(conn)?;

    let file_path = if input {
        format!("data/test_cases/{}/{}.in", problem_id, test_case_id)
    } else {
        format!("data/test_cases/{}/{}.out", problem_id, test_case_id)
    };

    Ok(NamedFile::open(file_path)?)
}
