#[macro_use]
extern crate rocket;

use rocket_dyn_templates::{context, Template};

#[get("/")]
fn home() -> Template {
    Template::render("home", context! {})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 54321,
            ..Default::default()
        })
        .mount("/", routes![home])
        .attach(Template::fairing())
}
