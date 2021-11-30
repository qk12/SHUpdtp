use diesel::prelude::*;
use server_core::errors::{ServiceError, ServiceResult};

pub fn get_self_type(region: String, db_connection: &PgConnection) -> ServiceResult<String> {
    use crate::schema::problem_sets as problem_sets_schema;
    if problem_sets_schema::table
        .filter(problem_sets_schema::region.eq(region.clone()))
        .count()
        .get_result::<i64>(db_connection)?
        == 1
    {
        return Ok("problem_set".to_string());
    }

    use crate::schema::contests as contests_schema;
    if contests_schema::table
        .filter(contests_schema::region.eq(region.clone()))
        .count()
        .get_result::<i64>(db_connection)?
        == 1
    {
        return Ok("contest".to_string());
    }

    let hint = "Not Found.".to_owned();
    Err(ServiceError::InternalServerErrorWithHint(hint))
}
