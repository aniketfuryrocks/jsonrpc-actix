use std::collections::HashMap;

use crate::types::{error::code::ErrorCode, params::Params, response::RpcPayload};
use futures_util::future::BoxFuture;
use futures_util::Future;

pub type RpcOutput = Result<RpcPayload, Box<dyn std::error::Error>>;

pub trait AsyncFn<Ctx: 'static> {
    fn call(&self, ctx: Ctx, params: Params) -> BoxFuture<'static, RpcOutput>;
}

impl<T, F, Ctx> AsyncFn<Ctx> for T
where
    T: Fn(Ctx, Params) -> F,
    F: Future<Output = RpcOutput> + 'static + Send,
    Ctx: 'static,
{
    fn call(&self, ctx: Ctx, params: Params) -> BoxFuture<'static, RpcOutput> {
        Box::pin(self(ctx, params))
    }
}

/// Stores a method to function map with ctx i.e state
pub struct RpcModule<Ctx: Clone> {
    ctx: Ctx,
    methods: HashMap<&'static str, Box<dyn AsyncFn<Ctx>>>,
}

impl<Ctx: Clone + Default + 'static> Default for RpcModule<Ctx> {
    fn default() -> Self {
        Self::new(Ctx::default())
    }
}

impl<Ctx: Clone + 'static> RpcModule<Ctx> {
    pub fn new(ctx: Ctx) -> Self {
        Self {
            ctx,
            methods: HashMap::new(),
        }
    }

    pub async fn call(&self, method: &str, params: Params) -> RpcOutput {
        let Some(call) = self.methods.get(method) else {
            return Ok(ErrorCode::MethodNotFound.into());
        };

        call.call(self.ctx.clone(), params).await
    }

    pub fn register<F>(&mut self, method: &'static str, call: F)
    where
        F: AsyncFn<Ctx> + Send + Sync + 'static,
        Ctx: 'static,
    {
        self.methods.insert(method, Box::new(call));
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
