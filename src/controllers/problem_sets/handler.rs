use crate::models::users::LoggedUser;
use crate::services::problem_set;
use actix_web::{delete, get, post, put, web, HttpResponse};
use server_core::database::Pool;
use server_core::errors::ServiceError;

#[derive(Deserialize)]
pub struct CreateProblemSetBody {
    region: String,
    title: String,
    introduction: Option<String>,
    can_view_testcases: bool,
}

#[post("")]
pub async fn create(
    body: web::Json<CreateProblemSetBody>,
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
        problem_set::create(
            body.region.clone(),
            body.title.clone(),
            body.introduction.clone(),
            body.can_view_testcases,
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
pub struct GetProblemSetListParams {
    title_filter: Option<String>,
    limit: i32,
    offset: i32,
}

#[get("")]
pub async fn get_set_list(
    query: web::Query<GetProblemSetListParams>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || {
        problem_set::get_set_list(query.title_filter.clone(), query.limit, query.offset, pool)
    })
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

    let res = web::block(move || problem_set::delete(region, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[derive(Deserialize)]
pub struct UpdateProblemSetBody {
    new_title: Option<String>,
    new_introduction: Option<String>,
    new_can_view_testcases: Option<bool>,
}

#[put("/{region}")]
pub async fn update(
    web::Path(region): web::Path<String>,
    body: web::Json<UpdateProblemSetBody>,
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
        problem_set::update(
            region.clone(),
            body.new_title.clone(),
            body.new_introduction.clone(),
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
