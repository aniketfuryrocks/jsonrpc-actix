use actix_web::{web, App, HttpServer};
use jsonrpc_actix::{
    handle::rpc_handler,
    methods::{RpcModule, RpcResult},
};

async fn get_version(_ctx: ()) -> RpcResult<u32> {
    Ok(1)
}

async fn foo(_ctx: (), count: Option<u32>, b: Option<u32>) -> RpcResult<&'static str> {
    println!("{count:?} {b:?}");

    Ok("bar")
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
