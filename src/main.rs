#[macro_use]
extern crate rocket;

use rocket::fs::FileServer;
use rocket_dyn_templates::{context, Template};

#[get("/")]
fn markdown_to_html() -> Template {
    Template::render("markdown_to_html", context! {})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 54321,
            ..Default::default()
        })
        .mount("/", routes![markdown_to_html])
        .mount("/static", FileServer::from("./static"))
        .attach(Template::fairing())
}
