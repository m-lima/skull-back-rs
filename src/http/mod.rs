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
            route.get("/").to(handler::List::new(|store| {
                Ok(store
                    .skull()
                    .list()
                    .map_err(error::Error::Store)?
                    .iter()
                    .map(|(id, skull)| (**id, (*skull).clone()))
                    .collect::<Vec<(store::Id, store::Skull)>>())

                // TODO: Fix this onwership problem. It's causing double copy (once for ownership
                // and again for serialization
                // store.skull().list().map_err(error::Error::Store)
            }));

            route.post("/").to_new_handler(handler::Create::new(yo));
            // route
            //     .post("/")
            //     .to_new_handler(handler::Create::new(|store, skull| {
            //         store.skull().create(skull).map_err(error::Error::Store)
            //     }));
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
