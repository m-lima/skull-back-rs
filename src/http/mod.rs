mod error;
mod handler;
mod mapper;
mod middleware;

use crate::options;
use crate::store;

pub fn route(options: options::Options) -> gotham::router::Router {
    use gotham::pipeline;
    use gotham::router::builder;
    use gotham::router::builder::{DefineSingleRoute, DrawRoutes};

    let pipeline = pipeline::new_pipeline()
        .add(middleware::Store::new(store::in_memory()))
        .add(middleware::Log)
        .build();

    let (chain, pipelines) = pipeline::single::single_pipeline(pipeline);

    builder::build_router(chain, pipelines, |route| {
        route.scope("/skull", |route| {
            route.get("/").to_new_handler(handler::List::new(|store| {
                Ok(store
                    .skull()
                    .list()
                    .map_err(error::Error::Store)?
                    .iter()
                    .map(|(id, skull)| (**id, (*skull).clone()))
                    .collect::<Vec<(store::Id, store::Skull)>>())
            }));

            route
                .post("/")
                .to_new_handler(handler::Create::new(|store, skull| {
                    store.skull().create(skull).map_err(error::Error::Store)
                }));
            // route
            //     .get("/:id:[0-9]+")
            //     .with_path_extractor::<mapper::request::Id>()
            //     .to(handler::skull::Read);
            route
                .put("/:id:[0-9]+")
                .with_path_extractor::<mapper::request::Id>()
                .to_new_handler(handler::Update::new(|store, id, skull| {
                    store.skull().update(id, skull).map_err(error::Error::Store)
                }));
            // route
            //     .delete("/:id:[0-9]+")
            //     .with_path_extractor::<mapper::request::Id>()
            //     .to(handler::skull::Delete);
        });
    })
}

// #[derive(Copy, Clone)]
// pub struct List<HandlerFunc>(HandlerFunc);

// impl<HandlerFunc> List<HandlerFunc>
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>,
// {
//     async fn handle(
//         self,
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
//         use gotham::state::FromState;

//         let json = (self.0)(middleware::Store::borrow_mut_from(state).get()?)?;

//         let response = gotham::hyper::Response::builder()
//             .header(gotham::hyper::header::CONTENT_TYPE, "application/json")
//             .header(
//                 gotham::helpers::http::header::X_REQUEST_ID,
//                 gotham::state::request_id::request_id(state),
//             )
//             .status(gotham::hyper::StatusCode::OK)
//             .body(gotham::hyper::Body::from(json))?;

//         Ok(response)
//     }
// }

// impl<HandlerFunc> List<HandlerFunc>
// where
//     HandlerFunc: Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>,
// {
//     pub fn new(handler_func: HandlerFunc) -> Self {
//         Self(handler_func)
//     }

//     async fn wrap(self, mut state: gotham::state::State) -> gotham::handler::HandlerResult {
//         match self.handle(&mut state).await {
//             Ok(r) => Ok((state, r)),
//             Err(e) => Err((state, e.into_handler_error())),
//         }
//     }
// }

// impl<HandlerFunc> gotham::handler::Handler for List<HandlerFunc>
// where
//     HandlerFunc: 'static
//         + Send
//         + Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>,
// {
//     fn handle(
//         self,
//         state: gotham::state::State,
//     ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
//         Box::pin(self.wrap(state))
//     }
// }

// impl<HandlerFunc> gotham::handler::NewHandler for List<HandlerFunc>
// where
//     HandlerFunc: 'static
//         + Copy
//         + Send
//         + Sync
//         + std::panic::RefUnwindSafe
//         + Fn(std::sync::MutexGuard<dyn store::Store>) -> Result<String, error::Error>,
// {
//     type Instance = Self;

//     fn new_handler(&self) -> gotham::anyhow::Result<Self::Instance> {
//         Ok(*self)
//     }
// }
