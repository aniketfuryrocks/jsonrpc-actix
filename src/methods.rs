use std::collections::HashMap;

use crate::types::{
    error::{code::ErrorCode, object::ErrorObject},
    params::Params,
    response::RpcPayload,
};
use futures_util::{future::BoxFuture, Future};
use serde::de::DeserializeOwned;

pub type RpcOutput = Result<RpcPayload, Box<dyn std::error::Error + Send>>;

pub trait AsyncFn<Ctx: 'static> {
    fn call(&self, ctx: Ctx, params: Params) -> Result<BoxFuture<'static, RpcOutput>, ErrorObject>;
}

impl<Func, Fut, Ctx> AsyncFn<Ctx> for Func
where
    Func: Fn(Ctx, Params) -> Result<Fut, ErrorObject>,
    Fut: Future<Output = RpcOutput> + 'static + Send,
    Ctx: 'static,
{
    fn call(&self, ctx: Ctx, params: Params) -> Result<BoxFuture<'static, RpcOutput>, ErrorObject> {
        if let Some(params) = &params {
            if !(params.is_array() || params.is_null()) {
                return Err(ErrorCode::InvalidRequest.into());
            }
        }
        Ok(Box::pin(self(ctx, params)?))
    }
}

pub trait IntoAsyncFn<Ctx: 'static, Args> {
    fn convert(self) -> Box<dyn AsyncFn<Ctx>>;
}

macro_rules! factor_tuple_inner {
    ($self: ident, $ctx: ident, $params: ident,) => {{
        if let Some(serde_json::Value::Array(arr)) = $params {
            if !arr.is_empty() {
                return Err(crate::types::error::object::ErrorObject::new(
                    crate::types::error::code::ErrorCode::InvalidParams,
                    crate::types::error::code::NO_PARAMS_EXPECTED_MSG,
                ));
            }
        }

        Ok(($self)($ctx))
    }};
    ($self: ident, $ctx: ident, $params: ident, $($param:ident,)+) => {{
        match serde_json::from_value($params.unwrap()) {
            Ok(($($param,)+)) => {
                Ok(($self)($ctx, $($param,)+))
            },
            Err(err) => {
                Err(crate::types::error::object::ErrorObject::new(
                    crate::types::error::code::ErrorCode::InvalidParams,
                    format!("Invalid params: {err}")
                ))
            }
        }
    }};
}

macro_rules! factory_tuple ({ $($param:ident)* } => {
    impl<Func, Fut, Ctx, $($param,)*> IntoAsyncFn<Ctx, ($($param,)*)> for Func
    where
        Func: Fn(Ctx, $($param),*) -> Fut + 'static,
        Fut: Future<Output = RpcOutput> + 'static + Send,
        Ctx: 'static,
        $($param:DeserializeOwned + 'static,)*
    {

        #[inline]
        #[allow(non_snake_case)]
        fn convert(self) -> Box<dyn AsyncFn<Ctx>> {
            Box::new(move |ctx: Ctx, params: Params| {
                factor_tuple_inner!(self,ctx, params, $($param,)*)
            })
        }
    }
});

factory_tuple! {}
factory_tuple! { A }
factory_tuple! { A B }
factory_tuple! { A B C }
factory_tuple! { A B C D }
factory_tuple! { A B C D E }
factory_tuple! { A B C D E F }
factory_tuple! { A B C D E F G }
factory_tuple! { A B C D E F G H }
factory_tuple! { A B C D E F G H I }
factory_tuple! { A B C D E F G H I J }
factory_tuple! { A B C D E F G H I J K }
factory_tuple! { A B C D E F G H I J K L }

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

        match call.call(self.ctx.clone(), params) {
            Ok(call) => call.await,
            Err(err) => Ok(RpcPayload::Error(err)),
        }
    }

    pub fn register<F, Args>(&mut self, method: &'static str, call: F)
    where
        F: IntoAsyncFn<Ctx, Args> + Send + Sync + 'static,
        Ctx: 'static,
    {
        self.methods.insert(method, call.convert());
    }
}
