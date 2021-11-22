#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;

mod auth;
mod controllers;
mod judge_actor;
mod models;
mod schema;
mod services;
mod statics;

use actix_cors::Cors;
use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer};

#[actix_web::get("/")]
async fn hello() -> impl actix_web::Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    // Get options
    let opt = {
        use structopt::StructOpt;
        server_core::cli_args::Opt::from_args()
    };

    let pool = server_core::database::pool::establish_connection_with_count(
        &opt.database_url,
        opt.judge_actor_count as u32 + 10,
    );
    let _domain = opt.domain.clone();
    let cookie_secret_key = opt.auth_secret_key.clone();
    let _secure_cookie = opt.secure_cookie;
    let auth_duration = time::Duration::hours(i64::from(opt.auth_duration_in_hour));

    let judge_actor_addr = judge_actor::start_judge_actor(opt.clone(), pool.clone());

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(judge_actor::JudgeActorAddr {
                addr: judge_actor_addr.clone(),
            })
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(cookie_secret_key.as_bytes())
                    .name("auth")
                    .path("/")
                    // .domain(&domain)
                    // Time from creation that cookie remains valid
                    .max_age_time(auth_duration)
                    // .same_site(actix_web::cookie::SameSite::None)
                    // Restricted to https?
                    .secure(false),
            ))
            .service(hello)
            .configure(controllers::users::route)
            .configure(controllers::problems::route)
            .configure(controllers::judge_servers::route)
            .configure(controllers::submissions::route)
            .configure(controllers::samples::route)
            .configure(controllers::regions::route)
            .configure(controllers::problem_sets::route)
            .configure(controllers::contests::route)
            .configure(controllers::announcements::route)
            .configure(controllers::problem_tags::route)
            .configure(controllers::groups::route)
    })
    .bind(("0.0.0.0", opt.port))
    .unwrap()
    .run()
    .await
}
