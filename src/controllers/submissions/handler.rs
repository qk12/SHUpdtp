use actix_web::{web, HttpResponse, get, post, put};
use crate::errors::ServiceError;
use crate::database::{ db_connection, Pool };
use crate::services::submission;
use crate::models::users::LoggedUser;
use crate::judge_actor::JudgeActorAddr;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateSubmissionBody {
    region: Option<String>,
    problem_id: i32,
    src: String,
    language: String,
}

#[post("")]
pub async fn create(
    body: web::Json<CreateSubmissionBody>,
    pool: web::Data<Pool>,
    logged_user: LoggedUser,
    judge_actor: web::Data<JudgeActorAddr>,
) -> Result<HttpResponse, ServiceError> {
    info!("{:?}", logged_user.0);
    if logged_user.0.is_none() { return Err(ServiceError::Unauthorized); }

    let res = web::block(move || submission::create(
        body.region.clone(),
        body.problem_id,
        logged_user.0.unwrap().id,
        body.src.clone(),
        body.language.clone(),
        pool,
        judge_actor,
    )).await.map_err(|e| {
        eprintln!("{}", e);
        e
    })?;

    Ok(HttpResponse::Ok().json(&res))
}

#[get("/{id}")]
pub async fn get(
    web::Path(submission_id): web::Path<Uuid>,
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if logged_user.0.is_none() { return Err(ServiceError::Unauthorized); }
    let cur_user = logged_user.0.unwrap();

    let conn = &db_connection(&pool)?;

    use crate::schema::submissions as submissions_schema;
    use diesel::prelude::*;

    let user_id: i32 = submissions_schema::table
        .filter(submissions_schema::id.eq(submission_id))
        .select(submissions_schema::user_id)
        .first(conn)?;

    if cur_user.id != user_id && cur_user.role != "super" && cur_user.role != "admin" {
        let hint = "No permission.".to_string();
        return  Err(ServiceError::BadRequest(hint));
    }

    let res = web::block(move || submission::get(
        submission_id,
        pool,
    )).await.map_err(|e| {
        eprintln!("{}", e);
        e
    })?;

    Ok(HttpResponse::Ok().json(&res))
}