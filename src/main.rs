use std::{fs::File, io::{BufReader, Read}, path::Path};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

//STRUCT -----------------------------------------------
#[derive(Deserialize)]
struct FormData {
    req_body: String,
}

//FUNCTIONS -----------------------------------------------


//ROUTES -----------------------------------------------
#[get("/")]
async fn hello() -> impl Responder {
    let file = File::open(
        Path::new("./html")
            .join("index")
            .with_extension("html"),
    ).expect("ERRORE NELLA LETTURA DEL FILE");

    let mut file_reader = BufReader::new(file);
    let mut html = String::new();
    
    let _a = file_reader.read_to_string(&mut html);

    HttpResponse::Ok().body(html)
}

#[post("/r")]
async fn response(form: web::Form<FormData>) -> HttpResponse {//////////////
    HttpResponse::Ok().body(format!("req_body: {}", form.req_body))
}
#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}



//MAIN -----------------------------------------------
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(response)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}