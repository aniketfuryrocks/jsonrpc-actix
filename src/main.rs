use actix_web::{web, App, HttpServer};
use jsonrpc_actix::{
    handle::rpc_handler,
    methods::{RpcModule, RpcOutput},
    types::response::RpcPayload,
};
use serde_json::json;

async fn get_version(_ctx: ()) -> RpcOutput {
    Ok(RpcPayload::Result(json!(0.1)))
}

async fn foo(_ctx: (), count: u32, b: u32) -> RpcOutput {
    println!("{count} {b}");
    Ok(RpcPayload::Result(json!("bar")))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let mut app_state = RpcModule::new(());
        app_state.register("foo", foo);
        app_state.register("getVersion", get_version);

        App::new()
            .app_data(web::Data::new(app_state))
            .service(web::resource("/").route(web::to(rpc_handler::<()>)))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
