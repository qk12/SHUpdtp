pub mod handler;

use actix_web::web;

pub fn route(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(handler::me)
            .service(handler::get_permitted_methods)
            .service(handler::get_name)
            .service(handler::get)
            .service(handler::create)
            .service(handler::update)
            .service(handler::get_list)
            .service(handler::login)
            .service(handler::logout)
            .service(handler::delete)
            .service(handler::get_submissions_count)
            .service(handler::get_submissions_time)
            .service(handler::upload_profile_picture)
            .service(handler::batch_create)
            .service(handler::reset_password)
            .service(handler::send_reset_password_token),
    );
}
