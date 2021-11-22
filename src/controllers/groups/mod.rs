pub mod handler;

use actix_web::web;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/groups")
            .service(handler::create)
            .service(handler::delete)
            .service(handler::update)
            .service(handler::get)
            .service(handler::get_list)
            .service(handler::insert_users)
            .service(handler::get_linked_user_column_list),
    );
}
