use crate::models::users::LoggedUser;
use crate::services::announcement;
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::*;
use server_core::database::Pool;
use server_core::errors::ServiceError;

#[derive(Deserialize)]
pub struct CreateAnnouncementBody {
    title: String,
    author: String,
    contents: String,
    release_time: Option<NaiveDateTime>,
}

#[post("")]
pub async fn create(
    body: web::Json<CreateAnnouncementBody>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }
    let cur_user = logged_user.0.unwrap();
    if cur_user.role != "sup" && cur_user.role != "admin" {
        let hint = "No permission.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    let res = web::block(move || {
        announcement::create(
            body.title.clone(),
            body.author.clone(),
            body.contents.clone(),
            body.release_time.clone(),
            pool,
        )
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        e
    })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[delete("/{id}")]
pub async fn delete(
    web::Path(id): web::Path<i32>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }
    let cur_user = logged_user.0.unwrap();
    if cur_user.role != "sup" && cur_user.role != "admin" {
        let hint = "No permission.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    let res = web::block(move || announcement::delete(id, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[derive(Deserialize)]
pub struct UpdateAnnouncementBody {
    new_title: Option<String>,
    new_author: Option<String>,
    new_contents: Option<String>,
    new_release_time: Option<NaiveDateTime>,
}

#[put("/{id}")]
pub async fn update(
    web::Path(id): web::Path<i32>,
    body: web::Json<UpdateAnnouncementBody>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }
    let cur_user = logged_user.0.unwrap();
    if cur_user.role != "sup" && cur_user.role != "admin" {
        let hint = "No permission.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    let res = web::block(move || {
        announcement::update(
            id,
            body.new_title.clone(),
            body.new_author.clone(),
            body.new_contents.clone(),
            body.new_release_time.clone(),
            pool,
        )
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        e
    })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[derive(Deserialize)]
pub struct GetAnnouncementListParams {
    id_filter: Option<i32>,
    title_filter: Option<String>,
    limit: i32,
    offset: i32,
    is_released: bool,
}

#[get("")]
pub async fn get_list(
    query: web::Query<GetAnnouncementListParams>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if !query.is_released.clone() {
        if logged_user.0.is_none() {
            return Err(ServiceError::Unauthorized);
        }
        let cur_user = logged_user.0.unwrap();
        if cur_user.role != "sup" && cur_user.role != "admin" {
            let hint = "No permission.".to_string();
            return Err(ServiceError::BadRequest(hint));
        }
    }

    let res = web::block(move || {
        announcement::get_list(
            query.id_filter,
            query.title_filter.clone(),
            query.limit,
            query.offset,
            query.is_released,
            pool,
        )
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        e
    })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[get("/{id}")]
pub async fn get(
    web::Path(id): web::Path<i32>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || announcement::get(id, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}
