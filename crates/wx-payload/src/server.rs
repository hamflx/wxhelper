use actix_web::{dev::Server, post, web::Data, App, HttpResponse, HttpServer, Responder};
use tokio::sync::mpsc::Sender;

use crate::wx::send_text_message;

pub fn start_http_server(sender: Sender<&'static str>) -> std::io::Result<Server> {
    let sender = Data::new(sender);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(sender.clone())
            .service(shutdown_api)
            .service(send_message_api)
    })
    .bind(("127.0.0.1", 9002))?
    .run();
    Ok(server)
}

#[post("/api/rt/shutdown")]
async fn shutdown_api(sender: Data<Sender<&'static str>>) -> impl Responder {
    let _ = sender.send("shutdown by api").await;
    HttpResponse::Ok().body("ret")
}

#[post("/api/wx/message")]
async fn send_message_api(req_body: String) -> impl Responder {
    let ret = match send_text_message(&req_body) {
        Ok(_) => "success",
        Err(_) => "fail",
    };
    HttpResponse::Ok().body(ret)
}
