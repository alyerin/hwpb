#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate bit_vec;
extern crate chrono;
extern crate csv;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[macro_use] extern crate error_chain;
extern crate itertools;
extern crate pam_auth;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

mod db;
mod errors;
mod user;
mod web;

use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let rocket = rocket::ignite();

    let database_url = rocket.config().get_str("database")
        .chain_err(|| "DATABASE_URL not set")?;

    // run any pending database migrations
    db::run_migrations(&database_url)?;

    // add current year on first run
    db::init_year(&database_url)?;

    rocket::ignite()
        .manage(db::init_pool(&database_url)?)
        .mount("/", routes![
            web::index,
            web::overview,
            web::event,
            web::group,
            web::static_file,
            web::session::nologin_index,
            web::session::nologin_path,
            web::session::login_redirect,
            web::session::get_login,
            web::session::post_login,
            web::session::logout,
        ])
        .mount("/api", routes![
            web::api::post_group,
            web::api::put_completion,
            web::api::delete_completion,
            web::api::put_elaboration,
            web::api::delete_elaboration,
            web::api::put_group_comment,
            web::api::put_group_desk,
            web::api::put_group_student,
            web::api::delete_group_student,
            web::api::search_groups,
            web::api::search_students,
            web::api::put_year,
            web::api::put_year_writable,
            web::api::post_experiment,
            web::api::delete_experiment,
            web::api::post_experiment_task,
            web::api::delete_experiment_task,
            web::api::put_event,
            web::api::delete_event,
            web::api::post_day,
            web::api::delete_day,
            web::api::post_student,
            web::api::post_students_csv,
            web::api::delete_student,
            web::api::post_tutor,
            web::api::delete_tutor,
            web::api::put_tutor_admin,
        ])
        .mount("/analysis", routes![
            web::analysis::passed,
            web::analysis::missing_reworks,
        ])
        .mount("/admin", routes![
            web::admin::index,
            web::admin::experiments,
            web::admin::events,
            web::admin::students,
            web::admin::tutors,
            web::admin::audit_index,
            web::admin::audit,
        ])
        .attach(rocket_contrib::Template::fairing())
        .attach(rocket::fairing::AdHoc::on_attach(|rocket| {
            match web::session::load_site_admins(rocket.config()) {
                Ok(site_admins) => Ok(rocket.manage(site_admins)),
                Err(error) => {
                    eprintln!("{}", error);
                    Err(rocket)
                }
            }
        }))
        .launch();

    Ok(())
}
