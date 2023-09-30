use std::{
    any::TypeId,
    collections::HashMap,
    sync::Arc,
    task::{Context, Poll},
};

use tokio::task::futures::TaskLocalFuture;
use tower::{Layer, Service};

use crate::{MapKey, SERVER_CONTEXT};

#[derive(Debug, Clone)]
pub struct ServerContextLayer<T: 'static + Send + Sync, I: 'static + Send + Sync, C: 'static + Send + Sync> {
    pub extentable_context: Arc<T>,
    pub internal_context: Arc<I>,
    pub config: Arc<C>
} // Internal + a open struct for other people

impl<S, T, I, C> Layer<S> for ServerContextLayer<T, I, C>
where
    T: 'static + Send + Sync,
    I: 'static + Send + Sync,
    C: 'static + Send + Sync,
{
    type Service = ServerContextService<S>;

    fn layer(&self, service: S) -> Self::Service {
        let mut context: HashMap<TypeId, MapKey> = HashMap::new();
        context.insert(TypeId::of::<T>(), self.extentable_context.clone());
        context.insert(TypeId::of::<I>(), self.internal_context.clone());
        context.insert(TypeId::of::<C>(), self.config.clone());
        ServerContextService {
            service,
            context: Arc::new(context),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerContextService<S> {
    service: S,
    context: Arc<HashMap<TypeId, MapKey>>,
}

impl<S, Request> Service<Request> for ServerContextService<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TaskLocalFuture<Arc<HashMap<TypeId, MapKey>>, S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        SERVER_CONTEXT.scope(self.context.clone(), self.service.call(request))
    }
}
