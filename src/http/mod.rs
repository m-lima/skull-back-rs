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
            route.get("/").to(handler::List::new(|_| Ok(String::new())));

            // route.post("/").to(handler::skull::Create);
            // route
            //     .get("/:id:[0-9]+")
            //     .with_path_extractor::<mapper::request::Id>()
            //     .to(handler::skull::Read);
            // route
            //     .put("/:id:[0-9]+")
            //     .with_path_extractor::<mapper::request::Id>()
            //     .to(handler::skull::Update);
            // route
            //     .delete("/:id:[0-9]+")
            //     .with_path_extractor::<mapper::request::Id>()
            //     .to(handler::skull::Delete);
        });
    })
}
