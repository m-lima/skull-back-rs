use crate::middleware;
use crate::options;
use crate::store;

pub fn route(options: options::Options) -> gotham::router::Router {
    use gotham::pipeline;
    use gotham::router::builder;
    use gotham::router::builder::{DefineSingleRoute, DrawRoutes};

    let pipeline = pipeline::new_pipeline()
        .add(middleware::Log)
        .add(middleware::Store::new(store::in_memory()))
        .build();

    let (chain, pipelines) = pipeline::single::single_pipeline(pipeline);

    builder::build_router(chain, pipelines, |route| {
        route
            .get("/:store/:id:[0-9]+")
            .with_path_extractor::<GetExtractor>()
            .to(get);

        route
            .get("/:store")
            .with_path_extractor::<GetAllPathExtractor>()
            .with_query_string_extractor::<GetAllQueryExtractor>()
            .to(get_all);
    })
}

fn get(
    mut state: gotham::state::State,
) -> (
    gotham::state::State,
    gotham::hyper::Response<gotham::hyper::Body>,
) {
    use gotham::handler::IntoResponse;
    use gotham::state::FromState;

    let get = GetExtractor::take_from(&mut state);

    let response = format!("Store: {}, Id: {}", get.store, get.id).into_response(&state);
    (state, response)
}

fn get_all(
    mut state: gotham::state::State,
) -> (
    gotham::state::State,
    gotham::hyper::Response<gotham::hyper::Body>,
) {
    use gotham::handler::IntoResponse;
    use gotham::state::FromState;

    let path = GetAllPathExtractor::take_from(&mut state);
    let query = GetAllQueryExtractor::take_from(&mut state);

    let response = format!("Store: {}, Limit: {:?}", path.store, query.limit).into_response(&state);
    (state, response)
}

#[derive(serde::Deserialize, gotham_derive::StateData, gotham_derive::StaticResponseExtender)]
pub struct GetExtractor {
    id: store::Id,
    store: Store,
}

#[derive(serde::Deserialize, gotham_derive::StateData, gotham_derive::StaticResponseExtender)]
pub struct GetAllPathExtractor {
    store: Store,
}

#[derive(serde::Deserialize, gotham_derive::StateData, gotham_derive::StaticResponseExtender)]
pub struct GetAllQueryExtractor {
    limit: Option<u32>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Store {
    Skull,
    Quick,
    Occurrence,
}

impl std::fmt::Display for Store {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Skull => f.write_str("Skull"),
            Self::Quick => f.write_str("Quick"),
            Self::Occurrence => f.write_str("Occurrence"),
        }
    }
}
