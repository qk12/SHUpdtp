pub mod handler;

use actix_web::web;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/problem_tags")
            .service(handler::create)
            //.service(handler::delete)
            .service(handler::update)
            .service(handler::get_list)
            .service(handler::apply_changes),
    );
}
