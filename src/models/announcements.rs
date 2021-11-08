use crate::schema::*;
use chrono::*;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct Announcement {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub contents: String,
    pub release_time: Option<NaiveDateTime>,
    pub last_update_time: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "announcements"]
pub struct InsertableAnnouncement {
    pub title: String,
    pub author: String,
    pub contents: String,
    pub release_time: Option<NaiveDateTime>,
    pub last_update_time: NaiveDateTime,
}

#[derive(AsChangeset)]
#[table_name = "announcements"]
pub struct AnnouncementForm {
    pub title: Option<String>,
    pub author: Option<String>,
    pub contents: Option<String>,
    pub release_time: Option<NaiveDateTime>,
    pub last_update_time: Option<NaiveDateTime>,
}

#[derive(Serialize)]
pub struct OutAnnouncement {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub contents: String,
    pub release_time: NaiveDateTime,
    pub last_update_time: NaiveDateTime,
}