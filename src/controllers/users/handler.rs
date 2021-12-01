use crate::models::users::{BatchCreateUserBody, LoggedUser};
use crate::services::user;
use actix_identity::Identity;
use actix_multipart::Multipart;
use actix_web::{delete, get, post, put, web, HttpResponse};
use futures::{StreamExt, TryStreamExt};
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
}

#[post("")]
pub async fn create(
    body: web::Json<CreateUserBody>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if body.role == "sup" || body.role == "admin" {
        if logged_user.0.is_none() {
            return Err(ServiceError::Unauthorized);
        }
        if logged_user.0.unwrap().role != "sup" {
            let hint = "No permission.".to_string();
            return Err(ServiceError::BadRequest(hint));
        }
    }

    let res = web::block(move || {
        user::create(
            body.username.clone(),
            body.password.clone(),
            body.email.clone(),
            body.role.clone(),
            body.real_name.clone(),
            body.school.clone(),
            body.student_number.clone(),
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

    if let Some(role) = &body.new_role {
        if role == "sup" || role == "admin" {
            if cur_user.role != "sup" {
                let hint = "No permission.".to_string();
                return Err(ServiceError::BadRequest(hint));
            }
        }
    }

    let res = web::block(move || {
        user::update(
            id,
            cur_user.role,
            body.new_username.clone(),
            body.new_password.clone(),
            body.new_email.clone(),
            body.new_role.clone(),
            body.new_real_name.clone(),
            body.new_school.clone(),
            body.new_student_number.clone(),
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
    username_or_email: String,
    password: String,
}

#[post("/login")]
pub async fn login(
    body: web::Json<LoginBody>,
    identity: Identity,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || {
        user::login(body.username_or_email.clone(), body.password.clone(), pool)
    })
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
    if cur_user.role != "sup" && cur_user.role != "admin" {
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

#[post("/{id}/profile_picture")]
pub async fn upload_profile_picture(
    web::Path(id): web::Path<i32>,
    logged_user: LoggedUser,
    mut payload: Multipart,
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

    let mut bytes = web::BytesMut::new();
    let mut filename = None;
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        if filename.is_none() {
            filename = Some(content_type.get_filename().unwrap().to_owned());
        } else {
            if filename.clone().unwrap() != content_type.get_filename().unwrap() {
                continue;
            }
        }

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            bytes.extend_from_slice(&data);
        }
    }

    let res = web::block(move || user::upload_profile_picture(id, filename.unwrap(), &bytes, pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[derive(Deserialize)]
pub struct BatchCreateUserList {
    list: Vec<BatchCreateUserBody>,
}

#[post("/batch_create")]
pub async fn batch_create(
    body: web::Json<BatchCreateUserList>,
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

    let res = web::block(move || user::batch_create(body.list.clone(), pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(()))
}

#[get("/reset_password/{email}")]
pub async fn send_reset_password_token(
    web::Path(email): web::Path<String>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res = web::block(move || user::send_reset_password_token(email.clone(), pool))
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[derive(Clone, Deserialize)]
pub struct ByOldPasswordBody {
    id: i32,
    old_password: String,
    new_password: String,
}

#[derive(Clone, Deserialize)]
pub struct ByEmailBody {
    email: String,
    token: String,
    new_password: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordBody {
    by_old_password: bool,
    by_old_password_body: Option<ByOldPasswordBody>,
    by_email: bool,
    by_email_body: Option<ByEmailBody>,
}

#[post("/reset_password")]
pub async fn reset_password(
    body: web::Json<ResetPasswordBody>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let res;

    if body.by_old_password {
        if logged_user.0.is_none() {
            return Err(ServiceError::Unauthorized);
        }
        let cur_user = logged_user.0.unwrap();
        let by_old_password_body = body.by_old_password_body.clone().unwrap();
        if cur_user.id != by_old_password_body.id {
            let hint = "No permission.".to_string();
            return Err(ServiceError::BadRequest(hint));
        }

        res = web::block(move || {
            user::reset_password_by_old_password(
                by_old_password_body.id,
                by_old_password_body.old_password.clone(),
                by_old_password_body.new_password.clone(),
                pool,
            )
        })
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;
    } else {
        let by_email_body = body.by_email_body.clone().unwrap();

        res = web::block(move || {
            user::reset_password_by_email_token(
                by_email_body.email,
                by_email_body.token,
                by_email_body.new_password,
                pool,
            )
        })
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            e
        })?;
    };

    Ok(HttpResponse::Ok().json(&res))
}
