use server_core::database::{db_connection, Pool};
use server_core::errors::ServiceResult;

use crate::models::problem_sets::*;

use crate::models::regions::*;
use crate::models::utils::SizedList;
use actix_web::web;
use diesel::prelude::*;

pub fn create(
    region: String,
    title: String,
    introduction: Option<String>,
    can_view_testcases: bool,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::regions as regions_schema;
    diesel::insert_into(regions_schema::table)
        .values(&Region {
            name: region.clone(),
            can_view_testcases: can_view_testcases,
        })
        .execute(conn)?;

    use crate::schema::problem_sets as problem_sets_schema;
    diesel::insert_into(problem_sets_schema::table)
        .values(&ProblemSetInfo {
            region: region,
            title: title,
            introduction: introduction,
        })
        .execute(conn)?;

    Ok(())
}

pub fn get_set_list(
    title_filter: Option<String>,
    limit: i32,
    offset: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<SizedList<ProblemSetInfo>> {
    let conn = &db_connection(&pool)?;

    let title_filter = if let Some(inner_data) = title_filter {
        Some(String::from("%") + &inner_data.as_str().replace(" ", "%") + "%")
    } else {
        None
    };

    use crate::schema::problem_sets as problem_sets_schema;
    let target = problem_sets_schema::table.filter(
        problem_sets_schema::title
            .nullable()
            .like(title_filter.clone())
            .or(title_filter.is_none()),
    );

    let total: i64 = target.clone().count().get_result(conn)?;

    let res = target
        .offset(offset.into())
        .limit(limit.into())
        .load(conn)?;

    Ok(SizedList {
        total: total,
        list: res,
    })
}

pub fn delete(region: String, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::regions as regions_schema;
    diesel::delete(regions_schema::table.filter(regions_schema::name.eq(region.clone())))
        .execute(conn)?;

    use crate::schema::problem_sets as problem_sets_schema;
    diesel::delete(
        problem_sets_schema::table.filter(problem_sets_schema::region.eq(region.clone())),
    )
    .execute(conn)?;

    use crate::schema::region_links as region_links_schema;
    diesel::delete(
        region_links_schema::table.filter(region_links_schema::region.eq(region.clone())),
    )
    .execute(conn)?;

    Ok(())
}

pub fn update(
    region: String,
    new_title: Option<String>,
    new_introduction: Option<String>,
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

    use crate::schema::problem_sets as problem_sets_schema;
    diesel::update(
        problem_sets_schema::table.filter(problem_sets_schema::region.eq(region.clone())),
    )
    .set(ProblemSetForm {
        title: new_title,
        introduction: new_introduction,
    })
    .execute(conn)?;

    Ok(())
}
