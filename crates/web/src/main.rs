extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::serve::StaticFiles;

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", StaticFiles::from("public"))
}

fn main() {
    rocket().launch();
}
