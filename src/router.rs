use crate::handler;
use crate::middleware;
use crate::options;
use crate::store;

// TODO: Move to main?
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
            route.get("/").to(handler::skull::List);

            route.post("/").to(handler::skull::Create);
            route
                .get("/:id:[0-9]+")
                .with_path_extractor::<IdExtractor>()
                .to(handler::skull::Read);
            route
                .put("/:id:[0-9]+")
                .with_path_extractor::<IdExtractor>()
                .to(handler::skull::Update);
            route
                .delete("/:id:[0-9]+")
                .with_path_extractor::<IdExtractor>()
                .to(handler::skull::Delete);
        });
    })
}

// TODO: Move to mapper? Have input extractors and output serializer there?
#[derive(serde::Deserialize, gotham_derive::StateData, gotham_derive::StaticResponseExtender)]
pub struct IdExtractor {
    id: store::Id,
}

impl IdExtractor {
    #[inline]
    pub fn id(self) -> store::Id {
        self.id
    }
}
