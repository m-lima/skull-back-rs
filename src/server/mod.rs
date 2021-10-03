mod error;
mod handler;
mod mapper;
mod middleware;

use crate::options;
use crate::store;

// Allowed because we can't create closures with moving the same data
#[allow(clippy::option_if_let_else)]
pub fn route(options: options::Options) -> anyhow::Result<gotham::router::Router> {
    let store = match options.store_path {
        Some(path) if path.is_dir() => middleware::Store::new(store::in_file(path, options.users)?),
        Some(path) => middleware::Store::new(store::in_file(path, options.users)?),
        None => middleware::Store::new(store::in_memory(options.users)),
    };

    if let Some(cors) = options.cors {
        Ok(with_cors(store, cors))
    } else {
        Ok(without_cors(store, options.web_path))
    }
}

fn with_cors(
    store: middleware::Store,
    cors: gotham::hyper::http::HeaderValue,
) -> gotham::router::Router {
    let pipeline = gotham::pipeline::new_pipeline()
        .add(store)
        .add(middleware::Log)
        .add(middleware::Cors::new(cors))
        .build();

    let (chain, pipelines) = gotham::pipeline::single::single_pipeline(pipeline);
    gotham::router::builder::build_router(chain, pipelines, |route| setup_resources(route))
}

fn without_cors(
    store: middleware::Store,
    web_path: Option<std::path::PathBuf>,
) -> gotham::router::Router {
    let pipeline = gotham::pipeline::new_pipeline()
        .add(store)
        .add(middleware::Log)
        .build();

    let (chain, pipelines) = gotham::pipeline::single::single_pipeline(pipeline);

    gotham::router::builder::build_router(chain, pipelines, |route| {
        use gotham::router::builder::{DefineSingleRoute, DrawRoutes};

        if let Some(web_path) = web_path {
            route.get("/").to_file(
                gotham::handler::assets::FileOptions::new(web_path.join("index.html")).build(),
            );
            route
                .get("/*")
                .to_dir(gotham::handler::assets::FileOptions::new(web_path).build());
            route.scope("/api", |route| setup_resources(route));
        } else {
            setup_resources(route);
        }
    })
}

pub fn setup_resources<C, P>(route: &mut impl gotham::router::builder::DrawRoutes<C, P>)
where
    C: gotham::pipeline::chain::PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: std::panic::RefUnwindSafe + Send + Sync + 'static,
{
    use gotham::router::builder::DefineSingleRoute;

    route.get("/modified").to(handler::LastModified);
    route.scope("/skull", Resource::<store::Skull>::setup);
    route.scope("/quick", Resource::<store::Quick>::setup);
    route.scope("/occurrence", Resource::<store::Occurrence>::setup);
}

struct Resource<D: store::CrudSelector>(std::marker::PhantomData<D>);

impl<D: store::CrudSelector> Resource<D> {
    pub fn setup<C, P>(route: &mut gotham::router::builder::ScopeBuilder<'_, C, P>)
    where
        C: gotham::pipeline::chain::PipelineHandleChain<P> + Copy + Send + Sync + 'static,
        P: std::panic::RefUnwindSafe + Send + Sync + 'static,
        D: 'static + Send + Sync + std::panic::RefUnwindSafe,
    {
        use gotham::router::builder::{DefineSingleRoute, DrawRoutes};

        route.get("/").to(handler::List::<D>::new());
        route.post("/").to(handler::Create::<D>::new());
        route
            .get("/:id:[0-9]+")
            .with_path_extractor::<mapper::request::Id>()
            .to(handler::Read::<D>::new());
        route
            .put("/:id:[0-9]+")
            .with_path_extractor::<mapper::request::Id>()
            .to(handler::Update::<D>::new());
        route
            .delete("/:id:[0-9]+")
            .with_path_extractor::<mapper::request::Id>()
            .to(handler::Delete::<D>::new());
    }
}
