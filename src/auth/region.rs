use crate::models::access_control_list::AccessControlListColumn;
use crate::models::users::LoggedUser;
use crate::services::group::utils::is_in_group;
use actix_web::web;
use diesel::prelude::*;
use server_core::database::{db_connection, Pool};
use server_core::errors::*;
use server_core::utils::time::get_cur_naive_date_time;

pub fn check_acl(conn: &PgConnection, user_id: i32, region: String) -> ServiceResult<()> {
    use crate::schema::access_control_list as access_control_list_schema;

    let acls: Vec<AccessControlListColumn> = access_control_list_schema::table
        .filter(access_control_list_schema::region.eq(region))
        .load(conn)?;

    for acl in acls {
        if acl.self_type == "user" {
            if acl.id == user_id {
                return Ok(());
            }
        } else {
            if is_in_group(conn, acl.id, user_id)? {
                return Ok(());
            }
        }
    }

    let hint = "Not in ACL.".to_owned();
    Err(ServiceError::UnauthorizedWithHint(hint))
}

pub fn is_manager(conn: &PgConnection, user_id: i32, region: String) -> ServiceResult<bool> {
    use crate::schema::access_control_list as access_control_list_schema;

    if access_control_list_schema::table
        .filter(access_control_list_schema::id.eq(user_id))
        .filter(access_control_list_schema::region.eq(region))
        .filter(access_control_list_schema::is_manager.eq(true))
        .count()
        .get_result::<i64>(conn)?
        == 1
    {
        Ok(true)
    } else {
        Ok(false)
    }
}

// have right to get colume to see problem list
pub fn check_view_right(
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
    region: String,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problem_sets as problem_sets_schema;
    if problem_sets_schema::table
        .filter(problem_sets_schema::region.eq(region.clone()))
        .count()
        .get_result::<i64>(conn)?
        == 1
    {
        return Ok(());
    }

    if let Some(user) = logged_user.0.clone() {
        if is_manager(conn, user.id, region.clone())? {
            return Ok(());
        }
    } else {
        return Err(ServiceError::Unauthorized);
    }

    use crate::models::contests;
    use crate::schema::contests as contests_schema;

    let contest = contests::Contest::from(
        contests_schema::table
            .filter(contests_schema::region.eq(region.clone()))
            .first::<contests::RawContest>(conn)?,
    );

    use contests::ContestState::*;
    match contests::get_contest_state(contest.clone(), get_cur_naive_date_time()) {
        Preparing => {
            if !contest.settings.view_before_start {
                let hint = "Contest do not allows viewing before start.".to_owned();
                return Err(ServiceError::UnauthorizedWithHint(hint));
            }
        }
        Ended => {
            if !contest.settings.view_after_end {
                let hint = "Contest do not allows viewing after end.".to_owned();
                return Err(ServiceError::UnauthorizedWithHint(hint));
            } else if contest.settings.public_after_end {
                return Ok(());
            }
        }
        _ => (),
    }

    if contest.hash.is_some() {
        if let Some(user) = logged_user.0 {
            check_acl(conn, user.id, region)
        } else {
            let hint = "Veiwing region which has access settings need to be logged in.".to_owned();
            return Err(ServiceError::UnauthorizedWithHint(hint));
        }
    } else {
        Ok(())
    }
}

// have right to see problem detail and submit in region
pub fn check_solve_right(
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
    region: String,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problem_sets as problem_sets_schema;
    if problem_sets_schema::table
        .filter(problem_sets_schema::region.eq(region.clone()))
        .count()
        .get_result::<i64>(conn)?
        == 1
    {
        return Ok(());
    }

    if let Some(user) = logged_user.0.clone() {
        if is_manager(conn, user.id, region.clone())? {
            return Ok(());
        }
    } else {
        return Err(ServiceError::Unauthorized);
    }

    use crate::models::contests;
    use crate::schema::contests as contests_schema;

    let contest = contests::Contest::from(
        contests_schema::table
            .filter(contests_schema::region.eq(region.clone()))
            .first::<contests::RawContest>(conn)?,
    );

    use contests::ContestState::*;
    match contests::get_contest_state(contest.clone(), get_cur_naive_date_time()) {
        Preparing => {
            let hint = "Contest do not allows visiting problems before start.".to_owned();
            return Err(ServiceError::UnauthorizedWithHint(hint));
        }
        Ended => {
            if !contest.settings.submit_after_end {
                let hint = "Contest do not allows visiting problems before start.".to_owned();
                return Err(ServiceError::UnauthorizedWithHint(hint));
            }
        }
        _ => (),
    }

    if contest.hash.is_some() {
        let user = logged_user.0.unwrap();
        check_acl(conn, user.id, region)
    } else {
        Ok(())
    }
}
