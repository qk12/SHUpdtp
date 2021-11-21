use crate::models::announcements::*;
use crate::models::utils::SizedList;
use actix_web::web;
use chrono::*;
use diesel::prelude::*;
use server_core::database::{db_connection, Pool};
use server_core::errors::{ServiceError, ServiceResult};
use server_core::utils::time::get_cur_naive_date_time;

pub fn create(
    title: String,
    author: String,
    contents: String,
    release_time: Option<NaiveDateTime>,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::announcements as announcements_schema;
    diesel::insert_into(announcements_schema::table)
        .values(&InsertableAnnouncement {
            title: title,
            author: author,
            contents: contents,
            release_time: release_time,
            last_update_time: get_cur_naive_date_time(),
        })
        .execute(conn)?;

    Ok(())
}

pub fn delete(id: i32, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::announcements as announcements_schema;
    diesel::delete(announcements_schema::table.filter(announcements_schema::id.eq(id)))
        .execute(conn)?;

    Ok(())
}

pub fn update(
    id: i32,
    new_title: Option<String>,
    new_author: Option<String>,
    new_contents: Option<String>,
    new_release_time: Option<NaiveDateTime>,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::announcements as announcements_schema;
    diesel::update(announcements_schema::table.filter(announcements_schema::id.eq(id)))
        .set(AnnouncementForm {
            title: new_title,
            author: new_author,
            contents: new_contents,
            release_time: new_release_time,
            last_update_time: Some(get_cur_naive_date_time()),
        })
        .execute(conn)?;

    Ok(())
}

pub fn get_list(
    id_filter: Option<i32>,
    title_filter: Option<String>,
    limit: i32,
    offset: i32,
    is_released: bool,
    pool: web::Data<Pool>,
) -> ServiceResult<SizedList<OutAnnouncement>> {
    let title_filter = if let Some(inner_data) = title_filter {
        Some(String::from("%") + &inner_data.as_str().replace(" ", "%") + "%")
    } else {
        None
    };

    let conn = &db_connection(&pool)?;

    use crate::schema::announcements as announcements_schema;
    let id_filter_predicate = announcements_schema::id
        .nullable()
        .eq(id_filter)
        .or(id_filter.is_none());
    let title_filter_predicate = announcements_schema::title
        .nullable()
        .like(title_filter.clone())
        .or(title_filter.is_none());
    let release_time_predicate = announcements_schema::release_time
        .is_not_null()
        .and(announcements_schema::release_time.le(get_cur_naive_date_time()));

    let total;
    let announcements;

    if is_released {
        let target = announcements_schema::table
            .filter(id_filter_predicate)
            .filter(title_filter_predicate.clone())
            .filter(release_time_predicate);

        total = target.clone().count().get_result(conn)?;

        announcements = target
            .offset(offset.into())
            .limit(limit.into())
            .order(announcements_schema::release_time.nullable().desc())
            .load::<Announcement>(conn)?;
    } else {
        let target = announcements_schema::table
            .filter(id_filter_predicate)
            .filter(title_filter_predicate);

        total = target.clone().count().get_result(conn)?;

        announcements = target
            .offset(offset.into())
            .limit(limit.into())
            .order(announcements_schema::release_time.nullable().desc())
            .load::<Announcement>(conn)?;
    };

    let mut out_announcements = Vec::new();
    for announcement in announcements {
        out_announcements.push(OutAnnouncement::from(announcement));
    }

    Ok(SizedList {
        total: total,
        list: out_announcements,
    })
}

pub fn get(id: i32, pool: web::Data<Pool>) -> ServiceResult<Announcement> {
    let conn = &db_connection(&pool)?;

    use crate::schema::announcements as announcements_schema;
    let announcement: Announcement = announcements_schema::table
        .filter(announcements_schema::id.eq(id))
        .first(conn)?;

    Ok(announcement)
}
