use crate::models::users::*;
use crate::models::utils::SizedList;
use actix_web::web;
use diesel::prelude::*;
use server_core::database::{db_connection, Pool};
use server_core::errors::{ServiceError, ServiceResult};
use server_core::utils::encryption;
use std::fs;
use std::io::prelude::*;

pub fn create(
    username: String,
    password: String,
    email: String,
    role: String,
    real_name: Option<String>,
    school: Option<String>,
    student_number: Option<String>,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let (salt, hash) = {
        let salt = encryption::make_salt();
        let hash = encryption::make_hash(&password, &salt).to_vec();
        (Some(salt), Some(hash))
    };

    let profile_picture = ProfilePicture {
        url: None,
        delete_url: None,
    };
    let profile_picture_string = serde_json::to_string(&profile_picture).unwrap();

    let conn = &db_connection(&pool)?;

    use crate::schema::users as users_schema;
    diesel::insert_into(users_schema::table)
        .values(&InsertableUser {
            salt: salt,
            hash: hash,
            username: username,
            email: email,
            role: role,
            real_name: real_name,
            school: school,
            student_number: student_number,
            profile_picture: profile_picture_string,
        })
        .execute(conn)?;

    Ok(())
}

pub fn get_name(id: i32, pool: web::Data<Pool>) -> ServiceResult<String> {
    let conn = &db_connection(&pool)?;

    use crate::schema::users as users_schema;

    let name: String = users_schema::table
        .filter(users_schema::id.eq(id))
        .select(users_schema::username)
        .first(conn)?;

    Ok(name)
}

pub fn get(id: i32, pool: web::Data<Pool>) -> ServiceResult<OutUser> {
    let conn = &db_connection(&pool)?;

    use crate::schema::users as users_schema;
    let user: User = users_schema::table
        .filter(users_schema::id.eq(id))
        .first(conn)?;

    Ok(OutUser::from(user))
}

pub fn update(
    id: i32,
    new_username: Option<String>,
    new_password: Option<String>,
    new_email: Option<String>,
    new_role: Option<String>,
    new_real_name: Option<String>,
    new_school: Option<String>,
    new_student_number: Option<String>,
    pool: web::Data<Pool>,
) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    let (new_salt, new_hash) = if let Some(inner_data) = new_password {
        let salt = encryption::make_salt();
        let hash = encryption::make_hash(&inner_data, &salt).to_vec();
        (Some(salt), Some(hash))
    } else {
        (None, None)
    };

    use crate::schema::users as users_schema;
    diesel::update(users_schema::table.filter(users_schema::id.eq(id)))
        .set(UserForm {
            salt: new_salt,
            hash: new_hash,
            username: new_username,
            email: new_email,
            role: new_role,
            real_name: new_real_name,
            school: new_school,
            student_number: new_student_number,
        })
        .execute(conn)?;

    Ok(())
}

pub fn get_list(
    id_filter: Option<i32>,
    username_filter: Option<String>,
    email_filter: Option<String>,
    role_filter: Option<String>,
    id_order: Option<bool>,
    limit: i32,
    offset: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<SizedList<OutUser>> {
    let username_filter = if let Some(inner_data) = username_filter {
        Some(String::from("%") + &inner_data.as_str().replace(" ", "%") + "%")
    } else {
        None
    };

    let email_filter = if let Some(inner_data) = email_filter {
        Some(String::from("%") + &inner_data.as_str().replace(" ", "%") + "%")
    } else {
        None
    };

    let conn = &db_connection(&pool)?;

    use crate::schema::users as users_schema;
    let target = users_schema::table
        .filter(
            users_schema::id
                .nullable()
                .eq(id_filter)
                .or(id_filter.is_none()),
        )
        .filter(
            users_schema::username
                .nullable()
                .like(username_filter.clone())
                .or(username_filter.is_none()),
        )
        .filter(
            users_schema::email
                .nullable()
                .like(email_filter.clone())
                .or(email_filter.is_none()),
        )
        .filter(
            users_schema::role
                .nullable()
                .eq(role_filter.clone())
                .or(role_filter.is_none()),
        );

    let total: i64 = target.clone().count().get_result(conn)?;

    let target = target.offset(offset.into()).limit(limit.into());

    let users: Vec<User> = match id_order {
        None => target.load(conn)?,
        Some(true) => target.order(users_schema::id.asc()).load(conn)?,
        Some(false) => target.order(users_schema::id.desc()).load(conn)?,
    };

    let out_users = {
        let mut res = Vec::new();
        for user in users {
            res.push(OutUser::from(user));
        }
        res
    };

    Ok(SizedList {
        total: total,
        list: out_users,
    })
}

pub fn login(username: String, password: String, pool: web::Data<Pool>) -> ServiceResult<SlimUser> {
    let conn = &db_connection(&pool)?;

    use crate::schema::users as users_schema;
    let user: User = users_schema::table
        .filter(users_schema::username.eq(username))
        .first(conn)?;

    if user.hash.is_none() || user.salt.is_none() {
        let hint = "Password was not set.".to_string();
        Err(ServiceError::BadRequest(hint))
    } else {
        let hash = encryption::make_hash(&password, &user.clone().salt.unwrap()).to_vec();
        if Some(hash) == user.hash {
            Ok(SlimUser::from(user))
        } else {
            let hint = "Password is wrong.".to_string();
            Err(ServiceError::BadRequest(hint))
        }
    }
}

pub fn get_permitted_methods(role: String, path: String) -> ServiceResult<Vec<String>> {
    use crate::statics::AUTH_CONFIG;
    match AUTH_CONFIG.get(&path) {
        Some(config) => match role.as_str() {
            "sup" => Ok(config.sup.clone().unwrap_or_default()),
            "admin" => Ok(config.admin.clone().unwrap_or_default()),
            "student" => Ok(config.student.clone().unwrap_or_default()),
            "teacher" => Ok(config.teacher.clone().unwrap_or_default()),
            _ => Ok(config.others.clone().unwrap_or_default()),
        },
        None => {
            let hint = "Path not set in config.".to_string();
            Err(ServiceError::BadRequest(hint))
        }
    }
}

pub fn delete(id: i32, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;
    use crate::schema::users as users_schema;

    let old_profile_picture_string = users_schema::table
        .filter(users_schema::id.eq(id))
        .select(users_schema::profile_picture)
        .first::<String>(conn)?;
    let old_profile_picture =
        serde_json::from_str::<ProfilePicture>(&old_profile_picture_string).unwrap();

    if let Some(old_delete_url) = old_profile_picture.delete_url {
        let client = reqwest::blocking::Client::new();
        let response = client.get(old_delete_url).send().unwrap();
    }

    diesel::delete(users_schema::table.filter(users_schema::id.eq(id))).execute(conn)?;

    Ok(())
}

pub fn get_submissions_count(
    user_id: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<UserSubmissionCount> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problems as problems_schema;
    use crate::schema::submissions as submissions_schema;

    let target = submissions_schema::table
        .filter(submissions_schema::user_id.eq(user_id))
        .inner_join(
            problems_schema::table.on(submissions_schema::problem_id.eq(problems_schema::id)),
        );

    let navie = target.filter(problems_schema::difficulty.lt(2.5));
    let easy = target.filter(
        problems_schema::difficulty
            .ge(2.5)
            .and(problems_schema::difficulty.lt(5.0)),
    );
    let middle = target.filter(
        problems_schema::difficulty
            .ge(5.0)
            .and(problems_schema::difficulty.lt(7.5)),
    );
    let hard = target.filter(problems_schema::difficulty.ge(7.5));

    let navie_submit_times: i64 = navie.count().get_result(conn)?;
    let navie_accept_times: i64 = navie
        .filter(submissions_schema::is_accepted.eq(true))
        .count()
        .get_result(conn)?;

    let easy_submit_times: i64 = easy.count().get_result(conn)?;
    let easy_accept_times: i64 = easy
        .filter(submissions_schema::is_accepted.eq(true))
        .count()
        .get_result(conn)?;

    let middle_submit_times: i64 = middle.count().get_result(conn)?;
    let middle_accept_times: i64 = middle
        .filter(submissions_schema::is_accepted.eq(true))
        .count()
        .get_result(conn)?;

    let hard_submit_times: i64 = hard.count().get_result(conn)?;
    let hard_accept_times: i64 = hard
        .filter(submissions_schema::is_accepted.eq(true))
        .count()
        .get_result(conn)?;

    let total_submit_times: i64 =
        navie_submit_times + easy_submit_times + middle_submit_times + hard_submit_times;
    let total_accept_times: i64 =
        navie_accept_times + easy_accept_times + middle_accept_times + hard_accept_times;

    Ok(UserSubmissionCount {
        total_submit_times: total_submit_times as i32,
        total_accept_times: total_accept_times as i32,
        navie_submit_times: navie_submit_times as i32,
        navie_accept_times: navie_accept_times as i32,
        easy_submit_times: easy_submit_times as i32,
        easy_accept_times: easy_accept_times as i32,
        middle_submit_times: middle_submit_times as i32,
        middle_accept_times: middle_accept_times as i32,
        hard_submit_times: hard_submit_times as i32,
        hard_accept_times: hard_accept_times as i32,
    })
}

pub fn get_submissions_time(
    user_id: i32,
    pool: web::Data<Pool>,
) -> ServiceResult<Vec<UserSubmissionTime>> {
    let conn = &db_connection(&pool)?;

    use crate::schema::submissions as submissions_schema;

    let raw_times: Vec<chrono::NaiveDateTime> = submissions_schema::table
        .filter(submissions_schema::user_id.eq(user_id))
        .select(submissions_schema::submit_time)
        .order(submissions_schema::submit_time.desc())
        .load(conn)?;

    let mut time_count: Vec<UserSubmissionTime> = Vec::new();
    let mut last = 0;
    let mut first: bool = true;
    for time in raw_times {
        if first {
            time_count.push(UserSubmissionTime {
                date: time.date(),
                count: 1,
            });
            first = false;
        } else if time.date() == time_count[last].date {
            time_count[last].count += 1;
        } else {
            time_count.push(UserSubmissionTime {
                date: time.date(),
                count: 1,
            });
            last += 1;
        }
    }

    Ok(time_count)
}

#[derive(Deserialize)]
pub struct SmmsProfilePictureData {
    file_id: i32,
    width: i32,
    height: i32,
    filename: String,
    storename: String,
    path: String,
    hash: String,
    url: String,
    delete: String,
    page: String,
}

#[derive(Deserialize)]
pub struct SmmsProfilePicture {
    success: bool,
    code: String,
    message: String,
    data: Option<SmmsProfilePictureData>,
    RequestId: String,
}

pub fn upload_profile_picture(
    id: i32,
    filename: String,
    picture_buf: &[u8],
    pool: web::Data<Pool>,
) -> ServiceResult<Option<String>> {
    let file_path = format!("image/tmp/{}", sanitize_filename::sanitize(&filename));

    let mut file = fs::File::create(file_path.clone())?;
    file.write_all(&picture_buf)
        .expect("Error writing picture.");

    let conn = &db_connection(&pool)?;
    use crate::schema::users as users_schema;

    let old_profile_picture_string = users_schema::table
        .filter(users_schema::id.eq(id))
        .select(users_schema::profile_picture)
        .first::<String>(conn)?;
    let old_profile_picture =
        serde_json::from_str::<ProfilePicture>(&old_profile_picture_string).unwrap();

    if let Some(old_delete_url) = old_profile_picture.delete_url {
        let client = reqwest::blocking::Client::new();
        let response = client.get(old_delete_url).send();
    }

    let client = reqwest::blocking::Client::new();
    let form = reqwest::blocking::multipart::Form::new().file("smfile", file_path.clone())?;

    let response_body: SmmsProfilePicture = client
        .post("https://sm.ms/api/v2/upload")
        .header("Authorization", "grM6s8VWsUrUDpcMkqIzPWsjCAJRe2E9")
        .multipart(form)
        .send()
        .unwrap()
        .json()
        .unwrap();

    fs::remove_file(file_path)?;

    if !response_body.success {
        let hint = response_body.message;
        return Err(ServiceError::BadRequest(hint));
    }

    let (new_url, new_delete_url) = {
        if let Some(inner_data) = response_body.data {
            (Some(inner_data.url), Some(inner_data.delete))
        } else {
            (None, None)
        }
    };

    let new_profile_picture = ProfilePicture {
        url: new_url.clone(),
        delete_url: new_delete_url,
    };
    let new_profile_picture_string = serde_json::to_string(&new_profile_picture).unwrap();

    diesel::update(users_schema::table.filter(users_schema::id.eq(id)))
        .set(users_schema::profile_picture.eq(new_profile_picture_string))
        .execute(conn)?;

    Ok(new_url)
}
