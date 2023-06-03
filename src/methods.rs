use std::{collections::HashMap, sync::Arc};

use crate::types::{error::code::ErrorCode, params::Params, response::RpcPayload};
use futures_util::future::BoxFuture;

pub type RpcOutput = Result<RpcPayload, Box<dyn std::error::Error>>;
pub type AsyncMethod<Ctx> = Arc<dyn Send + Sync + Fn(Params, Ctx) -> BoxFuture<'static, RpcOutput>>;

/// Stores a method to function map with ctx i.e state
pub struct RpcModule<Ctx: Clone> {
    ctx: Ctx,
    methods: HashMap<&'static str, AsyncMethod<Ctx>>,
}

impl<Ctx: Clone + Default> Default for RpcModule<Ctx> {
    fn default() -> Self {
        Self::new(Ctx::default())
    }
}

impl<Ctx: Clone> RpcModule<Ctx> {
    pub fn new(ctx: Ctx) -> Self {
        Self {
            ctx,
            methods: HashMap::new(),
        }
    }

    pub async fn call(&self, method: &str, params: Params) -> RpcOutput {
        let Some(call) = self.methods.get(method).cloned() else {
            return Ok(ErrorCode::MethodNotFound.into());
        };

        (call)(params, self.ctx.clone()).await
    }

    pub fn register(&mut self, method: &'static str, call: AsyncMethod<Ctx>) {
        self.methods.insert(method, call);
    }
}

#[tokio::test]
async fn test() {
    let module = RpcModule::default();

    async fn hello(params: Params) -> RpcOutput {
        Ok(RpcPayload::Result(serde_json::from_str("hi")))
    }

    module.register("hello", hello);

    let res = module.call("hello", Vec::with_capacity(0)).await;
    let expected = Ok(RpcPayload::Result(serde_json::from_str("hi")));

    assert_eq!(res, expected);
}
