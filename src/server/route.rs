use super::handler;
use super::mapper;
use super::middleware;
use crate::options;
use crate::store;

// Allowed because we can't create closures with moving the same data
#[allow(clippy::option_if_let_else)]
pub fn route(options: options::Options) -> anyhow::Result<gotham::router::Router> {
    let store = match options.store_path {
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

    let (chain, pipelines) = gotham::pipeline::single_pipeline(pipeline);
    gotham::router::builder::build_router(chain, pipelines, |route| {
        use gotham::router::builder::{DefineSingleRoute, DrawRoutes};

        route.options("/skull").to(|state| (state, ""));
        route.options("/skull/:id:[0-9]+").to(|state| (state, ""));
        route.options("/quick").to(|state| (state, ""));
        route.options("/quick/:id:[0-9]+").to(|state| (state, ""));
        route.options("/occurrence").to(|state| (state, ""));
        route
            .options("/occurrence/:id:[0-9]+")
            .to(|state| (state, ""));
        setup_resources(route);
    })
}

fn without_cors(
    store: middleware::Store,
    web_path: Option<std::path::PathBuf>,
) -> gotham::router::Router {
    let pipeline = gotham::pipeline::new_pipeline()
        .add(store)
        .add(middleware::Log)
        .build();

    let (chain, pipelines) = gotham::pipeline::single_pipeline(pipeline);

    gotham::router::builder::build_router(chain, pipelines, |route| {
        use gotham::router::builder::{DefineSingleRoute, DrawRoutes};

        if let Some(web_path) = web_path {
            route
                .get("/")
                .to_file(gotham::handler::FileOptions::new(web_path.join("index.html")).build());
            route
                .get("/*")
                .to_dir(gotham::handler::FileOptions::new(web_path).build());
            route.scope("/api", |route| setup_resources(route));
        } else {
            setup_resources(route);
        }
    })
}

pub fn setup_resources<C, P>(route: &mut impl gotham::router::builder::DrawRoutes<C, P>)
where
    C: gotham::pipeline::PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: std::panic::RefUnwindSafe + Send + Sync + 'static,
{
    route.scope("/skull", Resource::<store::Skull>::setup);
    route.scope("/quick", Resource::<store::Quick>::setup);
    route.scope("/occurrence", Resource::<store::Occurrence>::setup);
}

struct Resource<D: store::Selector>(std::marker::PhantomData<D>);

impl<D: store::Selector> Resource<D> {
    pub fn setup<C, P>(route: &mut gotham::router::builder::ScopeBuilder<'_, C, P>)
    where
        C: gotham::pipeline::PipelineHandleChain<P> + Copy + Send + Sync + 'static,
        P: std::panic::RefUnwindSafe + Send + Sync + 'static,
        D: 'static + Send + Sync + std::panic::RefUnwindSafe,
    {
        use gotham::router::builder::{DefineSingleRoute, DrawRoutes};

        route.head("/").to(handler::LastModified::<D>::new());
        route
            .get("/")
            .with_query_string_extractor::<mapper::request::Limit>()
            .to(handler::List::<D>::new());
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
