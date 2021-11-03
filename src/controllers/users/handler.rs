use crate::models::users::LoggedUser;
use crate::services::user;
use actix_identity::Identity;
use actix_web::{delete, get, post, put, web, HttpResponse};
use server_core::database::Pool;
use server_core::errors::ServiceError;

#[derive(Deserialize)]
pub struct GetUserListParams {
    id_filter: Option<i32>,
    username_filter: Option<String>,
    email_filter: Option<String>,
    role_filter: Option<String>,
    id_order: Option<bool>,
    limit: i32,
    offset: i32,
}

#[get("")]
pub async fn get_list(
    query: web::Query<GetUserListParams>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || {
        user::get_list(
            query.id_filter,
            query.username_filter.clone(),
            query.email_filter.clone(),
            query.role_filter.clone(),
            query.id_order.clone(),
            query.limit,
            query.offset,
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
pub struct CreateUserBody {
    username: String,
    password: String,
    email: String,
    role: String,
    real_name: Option<String>,
    school: Option<String>,
    student_number: Option<String>,
    profile_picture_url: Option<String>,
}

#[post("")]
pub async fn create(
    body: web::Json<CreateUserBody>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || {
        user::create(
            body.username.clone(),
            body.password.clone(),
            body.email.clone(),
            body.role.clone(),
            body.real_name.clone(),
            body.school.clone(),
            body.student_number.clone(),
            body.profile_picture_url.clone(),
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

#[get("/{id}/name")]
pub async fn get_name(
    web::Path(id): web::Path<i32>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || user::get_name(id, pool))
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
    let res = web::block(move || user::get(id, pool)).await.map_err(|e| {
        eprintln!("{}", e);
        e
    })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[derive(Deserialize)]
pub struct UpdateUserBody {
    new_username: Option<String>,
    new_password: Option<String>,
    new_email: Option<String>,
    new_role: Option<String>,
    new_real_name: Option<String>,
    new_school: Option<String>,
    new_student_number: Option<String>,
    new_profile_picture_url: Option<String>,
}

#[put("/{id}")]
pub async fn update(
    web::Path(id): web::Path<i32>,
    body: web::Json<UpdateUserBody>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }
    let cur_user = logged_user.0.unwrap();
    if cur_user.id != id && cur_user.role != "sup" && cur_user.role != "admin" {
        let hint = "No permission.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    let res = web::block(move || {
        user::update(
            id,
            body.new_username.clone(),
            body.new_password.clone(),
            body.new_email.clone(),
            body.new_role.clone(),
            body.new_real_name.clone(),
            body.new_school.clone(),
            body.new_student_number.clone(),
            body.new_profile_picture_url.clone(),
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
pub struct LoginBody {
    username: String,
    password: String,
}

#[post("/login")]
pub async fn login(
    body: web::Json<LoginBody>,
    identity: Identity,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || user::login(body.username.clone(), body.password.clone(), pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    let user_string = serde_json::to_string(&res).map_err(|_| ServiceError::InternalServerError)?;
    info!("user_string={}", user_string);
    identity.remember(user_string);
    Ok(HttpResponse::Ok().json(res))
}

#[post("/logout")]
pub fn logout(identity: Identity) -> HttpResponse {
    identity.forget();
    HttpResponse::Ok().finish()
}

#[get("/me")]
pub async fn me(logged_user: LoggedUser) -> Result<HttpResponse, ServiceError> {
    if let Some(res) = logged_user.0 {
        Ok(HttpResponse::Ok().json(&res))
    } else {
        Err(ServiceError::Unauthorized)
    }
}

#[derive(Deserialize)]
pub struct GetPermittedMethodsParams {
    path: String,
}

#[get("/permitted_methods")]
pub async fn get_permitted_methods(
    query: web::Query<GetPermittedMethodsParams>,
    logged_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }
    let cur_user = logged_user.0.unwrap();

    let res = web::block(move || user::get_permitted_methods(cur_user.role, query.path.clone()))
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
    if cur_user.id != id && cur_user.role != "sup" && cur_user.role != "admin" {
        let hint = "No permission.".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    let res = web::block(move || user::delete(id, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[get("/{id}/submissions_count")]
pub async fn get_submissions_count(
    web::Path(user_id): web::Path<i32>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }

    let res = web::block(move || user::get_submissions_count(user_id, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[get("/{id}/submissions_time")]
pub async fn get_submissions_time(
    web::Path(user_id): web::Path<i32>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() {
        return Err(ServiceError::Unauthorized);
    }

    let res = web::block(move || user::get_submissions_time(user_id, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}
