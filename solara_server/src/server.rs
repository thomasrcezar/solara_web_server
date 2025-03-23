use rocket::{get, routes};

#[get("/")]
fn index() -> &'static str {
    "Hello, world from Solara Server!"
}

#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub async fn start_server() -> Result<(), rocket::Error> {
    rocket::build()
        .mount("/", routes![index, hello])
        .launch()
        .await
        .map(|_| ()) // Convert Result<Rocket<Ignite>, Error> to Result<(), Error>
}
