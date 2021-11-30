use crate::auth::region::*;
use crate::models::contests::*;
use crate::models::users::LoggedUser;
use crate::services::contest;
use actix_web::{delete, get, post, put, web, HttpResponse};
use chrono::*;
use server_core::database::Pool;
use server_core::errors::ServiceError;

#[derive(Deserialize)]
pub struct CreateContestBody {
    region: String,
    title: String,
    introduction: Option<String>,
    self_type: String,
    start_time: NaiveDateTime,
    end_time: Option<NaiveDateTime>,
    seal_time: Option<NaiveDateTime>,
    settings: Option<ContestSettings>,
    password: Option<String>,
    can_view_testcases: bool,
}

#[post("")]
pub async fn create(
    body: web::Json<CreateContestBody>,
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
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
        contest::create(
            body.region.clone(),
            body.title.clone(),
            body.introduction.clone(),
            body.self_type.clone(),
            body.start_time.clone(),
            body.end_time.clone(),
            body.seal_time.clone(),
            if let Some(settings) = body.settings.clone() {
                settings
            } else {
                ContestSettings::default()
            },
            body.password.clone(),
            body.can_view_testcases,
            cur_user.id,
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
pub struct GetContestListParams {
    title_filter: Option<String>,
    include_ended: Option<bool>,
    limit: i32,
    offset: i32,
}

#[get("")]
pub async fn get_contest_list(
    query: web::Query<GetContestListParams>,
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || {
        contest::get_contest_list(
            query.title_filter.clone(),
            query.include_ended.unwrap_or(true),
            query.limit,
            query.offset,
            if let Some(user) = logged_user.0 {
                Some(user.id)
            } else {
                None
            },
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
pub struct RegisterToRegionBody {
    password: Option<String>,
}

#[post("/{region}/register")]
pub async fn register(
    web::Path(region): web::Path<String>,
    body: web::Json<RegisterToRegionBody>,
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }

    let res = web::block(move || {
        contest::register(
            region,
            body.password.clone(),
            logged_user.0.unwrap().id,
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

#[get("/{region}/rank_acm")]
pub async fn get_acm_rank(
    web::Path(region): web::Path<String>,
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    check_view_right(pool.clone(), logged_user.clone(), region.clone())?;

    let res = web::block(move || contest::get_acm_rank(region, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[delete("/{region}")]
pub async fn delete(
    web::Path(region): web::Path<String>,
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }
    let cur_user = logged_user.0.unwrap();
    if cur_user.role != "sup" && cur_user.role != "admin" {
        let hint = "No permission.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    let res = web::block(move || contest::delete(region, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[derive(Deserialize)]
pub struct UpdateContestBody {
    new_title: Option<String>,
    new_introduction: Option<String>,
    new_start_time: Option<NaiveDateTime>,
    new_end_time: Option<NaiveDateTime>,
    new_seal_time: Option<NaiveDateTime>,
    new_settings: Option<ContestSettings>,
    new_self_type: Option<String>,
    new_password: Option<String>,
    new_can_view_testcases: Option<bool>,
}

#[put("/{region}")]
pub async fn update(
    web::Path(region): web::Path<String>,
    body: web::Json<UpdateContestBody>,
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
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
        contest::update(
            region.clone(),
            body.new_title.clone(),
            body.new_introduction.clone(),
            body.new_start_time.clone(),
            body.new_end_time.clone(),
            body.new_seal_time.clone(),
            body.new_settings.clone(),
            body.new_self_type.clone(),
            body.new_password.clone(),
            body.new_can_view_testcases,
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

#[get("/{region}")]
pub async fn get(
    web::Path(region): web::Path<String>,
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || {
        contest::get(
            region.clone(),
            if let Some(user) = logged_user.0 {
                Some(user.id)
            } else {
                None
            },
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
pub struct InsertGroupIntoContestBody {
    group_ids: Vec<i32>,
}

#[post("/{region}/group")]
pub async fn insert_groups(
    web::Path(region): web::Path<String>,
    body: web::Json<InsertGroupIntoContestBody>,
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

    let res = web::block(move || contest::insert_groups(region, body.group_ids.clone(), pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[get("/{region}/group")]
pub async fn get_linked_groups(
    web::Path(region): web::Path<String>,
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

    let res = web::block(move || contest::get_linked_groups(region, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[delete("/{region}/group/{group_id}")]
pub async fn delete_group(
    web::Path((region, group_id)): web::Path<(String, i32)>,
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

    let res = web::block(move || contest::delete_group(region, group_id, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}
