use crate::schema::*;
use actix_identity::RequestIdentity;
use actix_web::dev::Payload;
use actix_web::{Error, FromRequest, HttpRequest};
use chrono::NaiveDate;
use shrinkwraprs::Shrinkwrap;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub salt: Option<String>,
    pub hash: Option<Vec<u8>>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub real_name: Option<String>,
    pub school: Option<String>,
    pub student_number: Option<String>,
    pub profile_picture: String,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct InsertableUser {
    pub salt: Option<String>,
    pub hash: Option<Vec<u8>>,
    pub username: String,
    pub email: String,
    pub role: String,
    pub real_name: Option<String>,
    pub school: Option<String>,
    pub student_number: Option<String>,
    pub profile_picture: String,
}

#[derive(Serialize)]
pub struct OutUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: String,
    pub real_name: Option<String>,
    pub school: Option<String>,
    pub student_number: Option<String>,
    pub profile_picture_url: Option<String>,
}

impl From<User> for OutUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            role: user.role,
            real_name: user.real_name,
            school: user.school,
            student_number: user.student_number,
            profile_picture_url: serde_json::from_str::<ProfilePicture>(&user.profile_picture)
                .unwrap()
                .url,
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "users"]
pub struct UserForm {
    pub salt: Option<String>,
    pub hash: Option<Vec<u8>>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub real_name: Option<String>,
    pub school: Option<String>,
    pub student_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlimUser {
    pub id: i32,
    pub role: String,
}

#[derive(Shrinkwrap, Clone, Default)]
pub struct LoggedUser(pub Option<SlimUser>);

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            role: user.role,
        }
    }
}

impl From<SlimUser> for LoggedUser {
    fn from(slim_user: SlimUser) -> Self {
        LoggedUser(Some(slim_user))
    }
}

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = futures::future::Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let identity = req.get_identity();

        let slim_user = if let Some(identity) = identity {
            match serde_json::from_str::<SlimUser>(&identity) {
                Err(e) => return futures::future::err(e.into()),
                Ok(y) => Ok(Some(y)),
            }
        } else {
            Ok(None)
        };

        futures::future::ready(slim_user.map(LoggedUser))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub sup: Option<Vec<String>>,
    pub admin: Option<Vec<String>>,
    pub teacher: Option<Vec<String>>,
    pub student: Option<Vec<String>>,
    pub net_friend: Option<Vec<String>>,
    pub others: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubmissionCount {
    pub total_submit_times: i32,
    pub total_accept_times: i32,
    pub navie_submit_times: i32,
    pub navie_accept_times: i32,
    pub easy_submit_times: i32,
    pub easy_accept_times: i32,
    pub middle_submit_times: i32,
    pub middle_accept_times: i32,
    pub hard_submit_times: i32,
    pub hard_accept_times: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSubmissionTime {
    pub date: NaiveDate,
    pub count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct ProfilePicture {
    pub url: Option<String>,
    pub delete_url: Option<String>,
}
