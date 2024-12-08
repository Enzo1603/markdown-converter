#[macro_use]
extern crate rocket;

use dotenvy::dotenv;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    rocket::build().mount("/", routes![index])
}
