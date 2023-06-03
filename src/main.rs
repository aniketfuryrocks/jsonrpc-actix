use actix_web::{web, App, HttpServer};
use jsonrpc_actix::methods::RpcOutput;
use jsonrpc_actix::types::response::RpcPayload;
use jsonrpc_actix::{handle::rpc_handler, methods::RpcModule, types::params::Params};

async fn foo(params: Params, _ctx: ()) -> RpcOutput {
    Ok(RpcPayload::Result(serde_json::from_str("bar").unwrap()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        let mut app_state = RpcModule::new(());
        app_state.register("foo", foo);

        App::new()
            .app_data(web::Data::new(app_state))
            .service(web::resource("/").route(web::to(rpc_handler::<()>)))
    })
    .bind(("127.0.0.1", 8080))
    .unwrap()
    .run()
    .await
}
