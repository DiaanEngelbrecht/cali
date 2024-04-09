use std::sync::Arc;

use tonic::transport::Server;

pub struct CaliConfig<T, Stack, ResultStack> {
    pub global_context: Option<Arc<T>>,
    pub database: bool,
    pub middleware_setup: Option<Box<dyn FnOnce(Server<Stack>) -> Server<ResultStack>>>,
}

impl<T, Stack, ResultStack> CaliConfig<T, Stack, ResultStack> {
    pub fn new() -> Self {
        Self {
            database: false,
            global_context: None,
            middleware_setup: None,
        }
    }

    /// By default cali doesn't try to connect to your database, call this on the CaliConfig to
    /// enable the database connection functionality of Cali
    pub fn enable_database(mut self) -> Self {
        self.database = true;

        self
    }

    /// Takes a function that provides the Cali Tower Stack, and should return a new Tower Stack.
    /// You can add all your Tower compliant middleware in here.
    pub fn add_middleware(
        mut self,
        setup_fn: impl FnOnce(Server<Stack>) -> Server<ResultStack> + 'static,
    ) -> Self {
        self.middleware_setup = Some(Box::new(setup_fn));

        self
    }

    // Allows the user of cali to make use of the global embedded context of cali. This is not to
    // be done without careful consideration of the consequences.
    pub fn add_global_context(mut self, global_context: T) -> Self {
        self.global_context = Some(Arc::new(global_context));

        self
    }
}
