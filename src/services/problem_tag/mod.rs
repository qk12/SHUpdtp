use crate::models::problem_tags::*;
use crate::models::utils::SizedList;
use crate::statics::PROBLEM_TAG_NAME_CACHE;
use actix_web::web;
use diesel::prelude::*;
use server_core::database::{db_connection, Pool};
use server_core::errors::{ServiceError, ServiceResult};

pub fn create(name: String, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problem_tags as problem_tags_schema;
    diesel::insert_into(problem_tags_schema::table)
        .values(&InsertableProblemTag { name: name })
        .execute(conn)?;

    Ok(())
}

pub fn delete(id: i32, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problems as problems_schema;
    if problems_schema::table
        .filter(problems_schema::tags.contains(vec![id]))
        .count()
        .get_result::<i64>(conn)?
        > 0
    {
        let hint = "标签已和题目关联，无法删除".to_string();
        return Err(ServiceError::BadRequest(hint));
    }

    use crate::schema::problem_tags as problem_tags_schema;
    diesel::delete(problem_tags_schema::table.filter(problem_tags_schema::id.eq(id)))
        .execute(conn)?;

    Ok(())
}

pub fn update(id: i32, new_name: Option<String>, pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problem_tags as problem_tags_schema;
    diesel::update(problem_tags_schema::table.filter(problem_tags_schema::id.eq(id)))
        .set(ProblemTagForm { name: new_name })
        .execute(conn)?;

    Ok(())
}

pub fn get_list(pool: web::Data<Pool>) -> ServiceResult<SizedList<ProblemTag>> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problem_tags as problem_tags_schema;
    let raw_tags = problem_tags_schema::table.load::<ProblemTag>(conn)?;

    Ok(SizedList {
        total: raw_tags.len() as i64,
        list: raw_tags,
    })
}

pub fn apply_changes(pool: web::Data<Pool>) -> ServiceResult<()> {
    let conn = &db_connection(&pool)?;

    use crate::schema::problem_tags as problem_tags_schema;
    let raw_tags = problem_tags_schema::table.load::<ProblemTag>(conn)?;

    {
        let mut lock = PROBLEM_TAG_NAME_CACHE.write().unwrap();
        lock.clear();

        for tags in &raw_tags {
            lock.insert(tags.id, tags.name.clone());
        }
    }

    Ok(())
}
