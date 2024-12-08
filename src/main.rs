#[macro_use]
extern crate rocket;

use dotenvy::dotenv;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 54321,
            ..Default::default()
        })
        .mount("/", routes![index])
}
