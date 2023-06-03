use actix_web::{web, App, HttpServer};
use jsonrpc_actix::{handle::rpc_handler, methods::RpcModule};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let app_state = RpcModule::new(());
        App::new()
            .app_data(web::Data::new(app_state))
            .service(web::resource("/").route(web::to(rpc_handler::<()>)))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
