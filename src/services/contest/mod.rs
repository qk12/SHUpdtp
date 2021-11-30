pub mod utils;

use crate::auth::region as region_access;
use crate::models::access_control_list::*;
use crate::models::contests::*;
use crate::models::groups::{Group, OutGroup};
use crate::models::ranks::*;
use crate::models::regions::*;
use crate::models::utils::SizedList;
use crate::services::rank::utils::update_acm_rank_cache;
use crate::statics::ACM_RANK_CACHE;
use actix_web::web;
use chrono::*;
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;
use server_core::database::{db_connection, Pool};
use server_core::errors::{ServiceError, ServiceResult};
use server_core::utils::encryption;
use server_core::utils::time::get_cur_naive_date_time;

pub fn create(
    region: String,
    title: String,
    introduction: Option<String>,
    self_type: String,
    start_time: NaiveDateTime,
    end_time: Option<NaiveDateTime>,
    seal_time: Option<NaiveDateTime>,
    settings: ContestSettings,
    password: Option<String>,
    can_view_testcases: bool,
    user_id: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    utils::check_settings_legal(settings.clone())?;

    let conn = &db_connection(&pool)?;

    use crate::schema::regions as regions_schema;
    diesel::insert_into(regions_schema::table)
        .values(&Region {
            name: region.clone(),
            can_view_testcases: can_view_testcases,
        })
        .execute(conn)?;

    let (salt, hash) = {
        if let Some(inner_data) = password {
            let salt = encryption::make_salt();
            let hash = encryption::make_hash(&inner_data, &salt).to_vec();
            (Some(salt), Some(hash))
        } else {
            (None, None)
        }
    };

    use crate::schema::contests as contests_schema;
    diesel::insert_into(contests_schema::table)
        .values(&RawContest {
            region: region.clone(),
            title: title,
            introduction: introduction,
            start_time: start_time,
            end_time: end_time,
            seal_time: seal_time,
            settings: serde_json::to_string(&settings).unwrap(),
            self_type: self_type,
            salt: salt,
            hash: hash,
        })
        .execute(conn)?;

    use crate::schema::access_control_list as access_control_list_schema;
    diesel::insert_into(access_control_list_schema::table)
        .values(&AccessControlListColumn {
            self_type: "user".to_string(),
            region,
            id: user_id,
            is_unrated: Some(true),
            is_manager: true,
        })
        .execute(conn)?;

    Ok(())
}

pub fn get_contest_list(
    title_filter: Option<String>,
    include_ended: bool,
    limit: i32,
    offset: i32,
    user_id: Option<i32>,
    pool: web::Data<Pool>,
) -> ServiceResult<SizedList<SlimContest>> {
    let conn = &db_connection(&pool)?;

    let title_filter = if let Some(inner_data) = title_filter {
        Some(String::from("%") + &inner_data.as_str().replace(" ", "%") + "%")
    } else {
        None
    };

    use crate::schema::contests as contests_schema;
    let target = contests_schema::table
        .filter(
            contests_schema::title
                .nullable()
                .like(title_filter.clone())
                .or(title_filter.is_none()),
        )
        .filter(
            contests_schema::end_time
                .lt(get_cur_naive_date_time())
                .or(contests_schema::end_time.is_null())
                .or(include_ended),
        );

    let total: i64 = target.clone().count().get_result(conn)?;

    let raw_contests = target
        .order(contests_schema::start_time.desc())
        .offset(offset.into())
        .limit(limit.into())
        .load::<RawContest>(conn)?;

    let mut res = Vec::new();
    for raw_contest in raw_contests {
        let mut t = SlimContest::from(raw_contest);

        if let Some(inner_data) = user_id {
            if region_access::check_acl(conn, inner_data, t.region.clone()).is_ok() {
                t.is_registered = true;
            }
        }

        res.push(t);
    }

    Ok(SizedList {
        total: total,
        list: res,
    })
}

pub fn register(
    region: String,
    maybe_password: Option<String>,
    user_id: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let mut is_unrated = Some(true);
    let conn = &db_connection(&pool)?;

    use crate::schema::contests as contests_schema;
    let contest = Contest::from(
        contests_schema::table
            .filter(contests_schema::region.eq(region.clone()))
            .first::<RawContest>(conn)?,
    );

    let contest_state = get_contest_state(contest.clone(), get_cur_naive_date_time());
    if contest_state == ContestState::Running || contest_state == ContestState::SealedRunning {
        if !contest.settings.register_after_start {
            let hint = "Contest not allows to register after start.".to_string();
            return Err(ServiceError::BadRequest(hint));
        } else if contest.settings.unrate_after_start {
            is_unrated = Some(false);
        }
    }

    if contest_state == ContestState::Ended && !contest.settings.public_after_end {
        let hint = "Contest not allows to register after end.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    if contest.hash.is_some() {
        if let Some(password) = maybe_password {
            let hash = encryption::make_hash(&password, &contest.salt.unwrap()).to_vec();
            if Some(hash) != contest.hash {
                let hint = "Password is wrong.".to_string();
                return Err(ServiceError::BadRequest(hint));
            }
        } else {
            let hint = "Password not given.".to_string();
            return Err(ServiceError::BadRequest(hint));
        }
    }

    use crate::schema::access_control_list as access_control_list_schema;
    diesel::insert_into(access_control_list_schema::table)
        .values(&AccessControlListColumn {
            self_type: "user".to_string(),
            region,
            id: user_id,
            is_unrated,
            is_manager: false,
        })
        .execute(conn)?;

    Ok(())
}

pub fn get_acm_rank(region: String, pool: web::Data<Pool>) -> ServiceResult<ACMRank> {
    let conn = &db_connection(&pool)?;

    use crate::schema::contests as contests_schema;
    let contest = Contest::from(
        contests_schema::table
            .filter(contests_schema::region.eq(region.clone()))
            .first::<RawContest>(conn)?,
    );

    let contest_state = get_contest_state(contest.clone(), get_cur_naive_date_time());
    let is_final = if contest_state == ContestState::Ended {
        true
    } else {
        false
    };

    let need_update = {
        let rank_cache = ACM_RANK_CACHE.read().unwrap();

        // not been refreshed in a minute
        if let Some(rank) = rank_cache.get(&region) {
            if get_cur_naive_date_time().timestamp() - rank.last_updated_time.timestamp() > 60 {
                true
            } else {
                false
            }
        } else {
            true
        }
    };

    if need_update {
        update_acm_rank_cache(region.clone(), conn, is_final)?;
    }

    Ok(ACM_RANK_CACHE
        .read()
        .unwrap()
        .get(&region)
        .unwrap()
        .to_owned())
}

pub fn delete(region: String, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::regions as regions_schema;
    diesel::delete(regions_schema::table.filter(regions_schema::name.eq(region.clone())))
        .execute(conn)?;

    use crate::schema::contests as contests_schema;
    diesel::delete(contests_schema::table.filter(contests_schema::region.eq(region.clone())))
        .execute(conn)?;

    use crate::schema::access_control_list as access_control_list_schema;
    diesel::delete(
        access_control_list_schema::table
            .filter(access_control_list_schema::region.eq(region.clone())),
    )
    .execute(conn)?;

    use crate::schema::region_links as region_links_schema;
    diesel::delete(
        region_links_schema::table.filter(region_links_schema::region.eq(region.clone())),
    )
    .execute(conn)?;

    ACM_RANK_CACHE.write().unwrap().remove(&region);

    Ok(())
}

pub fn update(
    region: String,
    new_title: Option<String>,
    new_introduction: Option<String>,
    new_start_time: Option<NaiveDateTime>,
    new_end_time: Option<NaiveDateTime>,
    new_seal_time: Option<NaiveDateTime>,
    new_settings: Option<ContestSettings>,
    new_self_type: Option<String>,
    new_password: Option<String>,
    new_can_view_testcases: Option<bool>,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::regions as regions_schema;
    diesel::update(regions_schema::table.filter(regions_schema::name.eq(region.clone())))
        .set(RegionForm {
            can_view_testcases: new_can_view_testcases,
        })
        .execute(conn)?;

    if let Some(settings) = new_settings.clone() {
        utils::check_settings_legal(settings)?;
    }

    let (new_salt, new_hash) = if let Some(inner_data) = new_password {
        let salt = encryption::make_salt();
        let hash = encryption::make_hash(&inner_data, &salt).to_vec();
        (Some(salt), Some(hash))
    } else {
        (None, None)
    };

    use crate::schema::contests as contests_schema;
    diesel::update(contests_schema::table.filter(contests_schema::region.eq(region.clone())))
        .set(ContestForm {
            title: new_title,
            introduction: new_introduction,
            start_time: new_start_time,
            end_time: new_end_time,
            seal_time: new_seal_time,
            settings: if let Some(inner_data) = new_settings {
                Some(serde_json::to_string(&inner_data).unwrap())
            } else {
                None
            },
            self_type: new_self_type,
            salt: new_salt,
            hash: new_hash,
        })
        .execute(conn)?;

    Ok(())
}

pub fn get(
    region: String,
    user_id: Option<i32>,
    pool: web::Data<Pool>,
) -> ServiceResult<OutContest> {
    let conn = &db_connection(&pool)?;

    use crate::schema::contests as contests_schema;
    let raw_contest: RawContest = contests_schema::table
        .filter(contests_schema::region.eq(region.clone()))
        .first(conn)?;

    let mut out_contest = OutContest::from(raw_contest);

    if let Some(inner_data) = user_id {
        if region_access::check_acl(conn, inner_data, out_contest.region.clone()).is_ok() {
            out_contest.is_registered = true;
        }
    }

    use crate::schema::region_links as region_links_schema;
    let problem_count = region_links_schema::table
        .filter(region_links_schema::region.eq(region.clone()))
        .count()
        .get_result::<i64>(conn)? as i32;

    out_contest.problem_count = problem_count;

    use crate::schema::access_control_list as access_control_list_schema;
    let registered_user_count = if out_contest.self_type == "open_contest" {
        access_control_list_schema::table
            .filter(access_control_list_schema::region.eq(region.clone()))
            .filter(access_control_list_schema::self_type.eq("user"))
            .count()
            .get_result::<i64>(conn)? as i32
    } else {
        let group_ids = access_control_list_schema::table
            .filter(access_control_list_schema::region.eq(region))
            .filter(access_control_list_schema::self_type.eq("group"))
            .select(access_control_list_schema::id)
            .load::<i32>(conn)?;

        use crate::schema::group_links as group_links_schema;
        group_links_schema::table
            .filter(group_links_schema::group_id.eq(any(group_ids)))
            .count()
            .get_result::<i64>(conn)? as i32
    };

    out_contest.registered_user_count = registered_user_count;

    Ok(out_contest)
}

pub fn insert_groups(
    region: String,
    group_ids: Vec<i32>,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::contests as contests_schema;
    let contest_type = contests_schema::table
        .filter(contests_schema::region.eq(region.clone()))
        .select(contests_schema::self_type)
        .first::<String>(conn)?;

    if contest_type != "group_contest" {
        let hint = "比赛类型不为小组赛.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    let mut res = Vec::new();
    for group_id in group_ids {
        res.push(AccessControlListColumn {
            self_type: "group".to_string(),
            id: group_id,
            region: region.clone(),
            is_unrated: Some(false),
            is_manager: false,
        });
    }

    use crate::schema::access_control_list as access_control_list_schema;
    diesel::insert_into(access_control_list_schema::table)
        .values(&res)
        .execute(conn)?;

    Ok(())
}

pub fn get_linked_groups(region: String, pool: web::Data<Pool>) -> ServiceResult<Vec<OutGroup>> {
    let conn = &db_connection(&pool)?;

    use crate::schema::access_control_list as access_control_list_schema;
    let group_ids = access_control_list_schema::table
        .filter(access_control_list_schema::region.eq(region))
        .filter(access_control_list_schema::self_type.eq("group"))
        .select(access_control_list_schema::id)
        .load::<i32>(conn)?;

    use crate::schema::groups as groups_schema;
    let groups = groups_schema::table
        .filter(groups_schema::id.eq(any(group_ids)))
        .load::<Group>(conn)?;

    let mut out_groups = Vec::new();
    for group in groups {
        out_groups.push(OutGroup::from(group));
    }

    Ok(out_groups)
}

pub fn delete_group(region: String, group_id: i32, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::contests as contests_schema;
    let contest_type = contests_schema::table
        .filter(contests_schema::region.eq(region.clone()))
        .select(contests_schema::self_type)
        .first::<String>(conn)?;

    if contest_type != "group_contest" {
        let hint = "比赛类型不为小组赛.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    use crate::schema::access_control_list as access_control_list_schema;
    diesel::delete(
        access_control_list_schema::table
            .filter(access_control_list_schema::region.eq(region))
            .filter(access_control_list_schema::self_type.eq("group"))
            .filter(access_control_list_schema::id.eq(group_id)),
    )
    .execute(conn)?;

    Ok(())
}
