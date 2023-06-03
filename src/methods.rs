use std::collections::HashMap;

use crate::types::{error::code::ErrorCode, params::Params, response::RpcPayload};
use futures_util::{future::BoxFuture, Future};
use serde::de::DeserializeOwned;

pub type RpcOutput = Result<RpcPayload, Box<dyn std::error::Error>>;

pub trait AsyncFn<Ctx: 'static> {
    fn call(&self, ctx: Ctx, params: Params) -> BoxFuture<'static, RpcOutput>;
}

impl<Func, Fut, Ctx> AsyncFn<Ctx> for Func
where
    Func: Fn(Ctx, Params) -> Fut,
    Fut: Future<Output = RpcOutput> + 'static + Send,
    Ctx: 'static,
{
    fn call(&self, ctx: Ctx, params: Params) -> BoxFuture<'static, RpcOutput> {
        Box::pin(self(ctx, params))
    }
}

pub trait IntoAsyncFn<Ctx: 'static, Args> {
    fn convert(self) -> Box<dyn AsyncFn<Ctx>>;
}

impl<Func, Fut, Ctx, A, B> IntoAsyncFn<Ctx, (A, B)> for Func
where
    Func: Fn(Ctx, A, B) -> Fut + 'static,
    Fut: Future<Output = RpcOutput> + 'static + Send,
    Ctx: 'static,
    A: DeserializeOwned + 'static,
    B: DeserializeOwned + 'static,
{
    fn convert(self) -> Box<dyn AsyncFn<Ctx>> {
        Box::new(move |ctx: Ctx, params: Params| {
            let mut params = params.unwrap().into_iter();
            let (A, B) = serde_json::from_value(params.next().unwrap()).unwrap();

            self(ctx, A, B)
        })
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

    pub fn register<F, Args>(&mut self, method: &'static str, call: F)
    where
        F: IntoAsyncFn<Ctx, Args> + Send + Sync + 'static,
        Ctx: 'static,
    {
        self.methods.insert(method, call.convert());
    }
}
