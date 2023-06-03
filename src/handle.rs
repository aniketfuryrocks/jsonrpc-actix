use actix_web::{web, Error, HttpResponse};
use bytes::Bytes;

use crate::{
    methods::RpcModule,
    types::{error::code::ErrorCode, request::RpcRequest, response::RpcResponse},
};

/// The main handler for JSONRPC server.
pub async fn rpc_handler<Ctx: Clone>(
    body: Bytes,
    app_state: web::Data<RpcModule<Ctx>>,
) -> Result<HttpResponse, Error> {
    let Ok(RpcRequest { jsonrpc, method, params, id  }) = serde_json::from_slice(body.as_ref()) else {
       return Ok(HttpResponse::Ok()
           .content_type("application/json")
           .body(RpcResponse {
            payload: ErrorCode::ParseError.into(),
            ..Default::default()
            }.dump())
        );
    };

    let payload = app_state.call(&method, params).await.expect("temprory");

    let result = RpcResponse {
        jsonrpc,
        id,
        payload,
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(result.dump()))
}
