use super::handler;
use super::mapper;
use super::middleware;
use crate::options;
use crate::store;

pub fn route(options: options::Options) -> anyhow::Result<gotham::router::Router> {
    macro_rules! cors {
        ($store: expr) => {
            if let Some(cors) = options.cors {
                Ok(with_cors($store, cors))
            } else {
                Ok(without_cors($store, options.web_path))
            }
        };
    }

    match options.db_path {
        Some(path) => cors!(middleware::Store::new(store::in_db(path, options.users)?)),
        None => match options.store_path {
            Some(path) => cors!(middleware::Store::new(store::in_file(path, options.users)?)),
            None => cors!(middleware::Store::new(store::in_memory(options.users))),
        },
    }
}

fn with_cors<S: store::Store>(
    store: middleware::Store<S>,
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
        setup_resources::<S, _, _>(route);
    })
}

fn without_cors<S: store::Store>(
    store: middleware::Store<S>,
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
            route.scope("/api", |route| setup_resources::<S, _, _>(route));
        } else {
            setup_resources::<S, _, _>(route);
        }
    })
}

fn setup_resources<S, C, P>(route: &mut impl gotham::router::builder::DrawRoutes<C, P>)
where
    S: store::Store,
    C: gotham::pipeline::PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: std::panic::RefUnwindSafe + Send + Sync + 'static,
{
    route.scope("/skull", Resource::<S, store::Skull>::setup);
    route.scope("/quick", Resource::<S, store::Quick>::setup);
    route.scope("/occurrence", Resource::<S, store::Occurrence>::setup);
}

struct Resource<S: store::Store, M: store::Model>(
    std::marker::PhantomData<S>,
    std::marker::PhantomData<M>,
);

impl<S: store::Store, M: store::Model> Resource<S, M>
where
    M: 'static + Send + Sync + std::panic::RefUnwindSafe,
{
    fn setup<C, P>(route: &mut gotham::router::builder::ScopeBuilder<'_, C, P>)
    where
        C: gotham::pipeline::PipelineHandleChain<P> + Copy + Send + Sync + 'static,
        P: std::panic::RefUnwindSafe + Send + Sync + 'static,
    {
        use gotham::router::builder::{DefineSingleRoute, DrawRoutes};

        route.head("/").to(handler::LastModified::<S, M>::new());
        route
            .get("/")
            .with_query_string_extractor::<mapper::request::Limit>()
            .to(handler::List::<S, M>::new());
        route.post("/").to(handler::Create::<S, M>::new());
        route
            .get("/:id:[0-9]+")
            .with_path_extractor::<mapper::request::Id>()
            .to(handler::Read::<S, M>::new());
        route
            .put("/:id:[0-9]+")
            .with_path_extractor::<mapper::request::Id>()
            .to(handler::Update::<S, M>::new());
        route
            .delete("/:id:[0-9]+")
            .with_path_extractor::<mapper::request::Id>()
            .to(handler::Delete::<S, M>::new());
    }
}
