pub mod utils;

use crate::models::group_links::*;
use crate::models::groups::*;
use crate::models::users::{OutUser, User};
use crate::models::utils::SizedList;
use actix_web::web;
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;
use server_core::database::{db_connection, Pool};
use server_core::errors::ServiceResult;

pub fn create(
    title: String,
    introduction: Option<String>,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::groups as groups_schema;
    diesel::insert_into(groups_schema::table)
        .values(&InsertableGroup {
            title: title,
            introduction: introduction,
        })
        .execute(conn)?;

    Ok(())
}

pub fn delete(id: i32, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::groups as groups_schema;
    diesel::delete(groups_schema::table.filter(groups_schema::id.eq(id))).execute(conn)?;

    Ok(())
}

pub fn update(
    id: i32,
    new_title: Option<String>,
    new_introduction: Option<String>,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::groups as groups_schema;
    diesel::update(groups_schema::table.filter(groups_schema::id.eq(id)))
        .set(GroupForm {
            title: new_title,
            introduction: new_introduction,
        })
        .execute(conn)?;

    Ok(())
}

pub fn get(id: i32, pool: web::Data<Pool>) -> ServiceResult<Group> {
    let conn = &db_connection(&pool)?;

    use crate::schema::groups as groups_schema;
    let group: Group = groups_schema::table
        .filter(groups_schema::id.eq(id))
        .first(conn)?;

    Ok(group)
}

pub fn get_list(
    title_filter: Option<String>,
    limit: i32,
    offset: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<SizedList<OutGroup>> {
    let title_filter = if let Some(inner_data) = title_filter {
        Some(String::from("%") + &inner_data.as_str().replace(" ", "%") + "%")
    } else {
        None
    };

    let conn = &db_connection(&pool)?;

    use crate::schema::groups as groups_schema;
    let target = groups_schema::table.filter(
        groups_schema::title
            .nullable()
            .like(title_filter.clone())
            .or(title_filter.is_none()),
    );

    let total: i64 = target.clone().count().get_result(conn)?;

    let groups = target
        .offset(offset.into())
        .limit(limit.into())
        .load::<Group>(conn)?;

    let mut out_groups = Vec::new();
    for group in groups {
        out_groups.push(OutGroup::from(group));
    }

    Ok(SizedList {
        total: total,
        list: out_groups,
    })
}

pub fn insert_users(
    id: i32,
    user_ids: Vec<i32>,
    pool: web::Data<Pool>,
) -> ServiceResult<Vec<InsertUserIntoGroupResult>> {
    let conn = &db_connection(&pool)?;

    use crate::schema::group_links as group_links_schema;

    let mut res = Vec::new();
    for user_id in user_ids {
        match diesel::insert_into(group_links_schema::table)
            .values(&GroupLink {
                group_id: id,
                user_id: user_id,
            })
            .execute(conn)
        {
            Ok(_) => {
                res.push(InsertUserIntoGroupResult {
                    user_id: user_id,
                    is_success: true,
                });
            }
            Err(_) => {
                res.push(InsertUserIntoGroupResult {
                    user_id: user_id,
                    is_success: false,
                });
            }
        }
    }

    Ok(res)
}

pub fn get_linked_user_column_list(
    id: i32,
    limit: i32,
    offset: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<SizedList<OutUser>> {
    let conn = &db_connection(&pool)?;

    use crate::schema::group_links as group_links_schema;
    let target = group_links_schema::table.filter(group_links_schema::group_id.eq(id));

    let total: i64 = target.clone().count().get_result(conn)?;

    let user_ids = target
        .select(group_links_schema::user_id)
        .offset(offset.into())
        .limit(limit.into())
        .load::<i32>(conn)?;

    use crate::schema::users as users_schema;
    let users = users_schema::table
        .filter(users_schema::id.eq(any(user_ids)))
        .load::<User>(conn)?;

    let mut out_users = Vec::new();
    for user in users {
        out_users.push(OutUser::from(user));
    }

    Ok(SizedList {
        total: total,
        list: out_users,
    })
}

pub fn delete_user(id: i32, user_id: i32, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::group_links as group_links_schema;
    diesel::delete(
        group_links_schema::table
            .filter(group_links_schema::group_id.eq(id))
            .filter(group_links_schema::user_id.eq(user_id)),
    )
    .execute(conn)?;

    Ok(())
}
