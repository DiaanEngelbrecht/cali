use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
    task::{Context, Poll},
};

use tokio::task::futures::TaskLocalFuture;
use tower::{Layer, Service};

use crate::{ServerContext, SERVER_CONTEXT};

#[derive(Debug, Clone)]
pub struct ServerContextLayer<T: 'static + Send + Sync> {
    pub extentable_context: Arc<T>,
    pub internal_context: Arc<ServerContext>,
} // Internal + a open struct for other people

impl<S, T> Layer<S> for ServerContextLayer<T>
where
    T: 'static + Send + Sync,
{
    type Service = ServerContextService<S>;

    fn layer(&self, service: S) -> Self::Service {
        let mut context: HashMap<TypeId, Arc<dyn Any + Send + Sync>> = HashMap::new();
        context.insert(TypeId::of::<T>(), self.extentable_context.clone());
        context.insert(TypeId::of::<ServerContext>(), self.internal_context.clone());
        ServerContextService {
            service,
            context: Arc::new(context),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerContextService<S> {
    service: S,
    context: Arc<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
}

impl<S, Request> Service<Request> for ServerContextService<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        TaskLocalFuture<Arc<HashMap<TypeId, Arc<(dyn Any + Send + Sync + 'static)>>>, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        SERVER_CONTEXT.scope(self.context.clone(), self.service.call(request))
    }
}
