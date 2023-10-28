mod auth;
mod logger;

#[derive(Debug, Copy, Clone)]
pub struct Logger;

#[derive(Debug, Clone)]
pub struct Auth<S> {
    services: std::sync::Arc<std::collections::HashMap<String, S>>,
}

impl<S> Auth<S> {
    pub fn wrap(services: std::collections::HashMap<String, S>) -> Self {
        Self {
            services: std::sync::Arc::new(services),
        }
    }
}
