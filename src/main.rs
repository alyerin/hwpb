#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate chrono;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate dotenv;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

pub mod db;
pub mod user;
pub mod web;

use dotenv::dotenv;

use db::*;

fn main() {
    dotenv().ok();

    rocket::ignite()
        .manage(db::init_pool())
        .mount("/", routes![
            web::index,
            web::event,
            web::static_file,
            web::session::index,
            web::session::get_login,
            web::session::post_login,
            web::session::logout,
        ])
        .mount("/api", routes![
            web::api::put_completion,
            web::api::delete_completion,
            web::api::put_elaboration,
            web::api::delete_elaboration,
            web::api::put_comment,
            web::api::put_group_student,
            web::api::delete_group_student,
        ])
        .attach(rocket_contrib::Template::fairing())
        .attach(rocket::fairing::AdHoc::on_attach(|rocket| {
            let allowed_users = {
                let users = rocket.config().get_slice("allowed_users").unwrap_or(&[]);
                let users = users.iter().filter_map(|u| u.as_str());
                web::session::AllowedUsers::new(users)
            };
            Ok(rocket.manage(allowed_users))
        }))
        .launch();
}
