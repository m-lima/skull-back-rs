// TODO: Rewrite once try-blocks lands https://github.com/rust-lang/rust/issues/31436
#[proc_macro_derive(Handler)]
pub fn handler(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let syn::DeriveInput {
        ident, generics, ..
    } = syn::parse_macro_input!(input);

    let types = generics.type_params().collect::<Vec<_>>();

    // if types.len() != 2 {
    //     panic!("Can only have one param");
    // }

    let handler_func = types[0].clone();
    let data = types[1].clone();

    let where_clause = generics
        .where_clause
        .expect("The handler function must be bound");

    let output = quote::quote! {
        impl<#handler_func, #data> #ident<#handler_func, #data>
        #where_clause
        {
            pub fn new(handler_func: #handler_func) -> Self {
                Self(handler_func, std::marker::PhantomData::default())
            }

            async fn wrap(self, mut state: gotham::state::State) -> gotham::handler::HandlerResult {
                match self.handle(&mut state).await {
                    Ok(r) => Ok((state, r)),
                    Err(e) => Err((state, e.into_handler_error())),
                }
            }
        }

        impl<#handler_func, #data> gotham::handler::Handler for #ident<#handler_func, #data>
        #where_clause + 'static + Send, #data: 'static + Send,
        {
            fn handle(
                self,
                state: gotham::state::State,
            ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
                Box::pin(self.wrap(state))
            }
        }

        impl<#handler_func, #data> gotham::handler::NewHandler for #ident<#handler_func, #data>
        #where_clause + 'static + Copy + Send + Sync + std::panic::RefUnwindSafe,
        #data + 'static + Copy + Send + Sync + std::panic::RefUnwindSafe
        {
            type Instance = Self;

            fn new_handler(&self) -> gotham::anyhow::Result<Self::Instance> {
                Ok(*self)
            }
        }
    };

    output.into()
}

// Reference
// pub struct Create<HandlerFunc, Data>(HandlerFunc, std::marker::PhantomData<Data>)
// where
//     HandlerFunc: Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error>,
//     Data: store::Data;

// impl<HandlerFunc, Data> Clone for Create<HandlerFunc, Data>
// where
//     HandlerFunc: Copy + Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error>,
//     Data: store::Data,
// {
//     fn clone(&self) -> Self {
//         Self(self.0, self.1)
//     }
// }

// impl<HandlerFunc, Data> Copy for Create<HandlerFunc, Data>
// where
//     HandlerFunc: Copy + Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error>,
//     Data: store::Data,
// {
// }

// impl<HandlerFunc, Data> Create<HandlerFunc, Data>
// where
//     HandlerFunc: Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error>,
//     Data: store::Data,
// {
//     async fn handle(
//         self,
//         state: &mut gotham::state::State,
//     ) -> Result<gotham::hyper::Response<gotham::hyper::Body>, error::Error> {
//         use gotham::state::FromState;

//         let body = mapper::request::body(state).await?;

//         let id = (self.0)(&mut *middleware::Store::borrow_mut_from(state).get()?, body)?;

//         let response = gotham::hyper::Response::builder()
//             .header(gotham::hyper::header::LOCATION, id)
//             .header(
//                 gotham::helpers::http::header::X_REQUEST_ID,
//                 gotham::state::request_id::request_id(state),
//             )
//             .status(gotham::hyper::StatusCode::CREATED)
//             .body(gotham::hyper::Body::empty())?;

//         Ok(response)
//     }
// }

// impl<HandlerFunc, Data> Create<HandlerFunc, Data>
// where
//     HandlerFunc: Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error>,
//     Data: store::Data,
// {
//     pub fn new(handler_func: HandlerFunc) -> Self {
//         Self(handler_func, std::marker::PhantomData::default())
//     }

//     async fn wrap(self, mut state: gotham::state::State) -> gotham::handler::HandlerResult {
//         match self.handle(&mut state).await {
//             Ok(r) => Ok((state, r)),
//             Err(e) => Err((state, e.into_handler_error())),
//         }
//     }
// }

// impl<HandlerFunc, Data> gotham::handler::Handler for Create<HandlerFunc, Data>
// where
//     HandlerFunc:
//         Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error> + 'static + Send,
//     Data: store::Data + 'static + Send,
// {
//     fn handle(
//         self,
//         state: gotham::state::State,
//     ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
//         Box::pin(self.wrap(state))
//     }
// }

// impl<HandlerFunc, Data> gotham::handler::NewHandler for Create<HandlerFunc, Data>
// where
//     Self: Copy,
//     HandlerFunc: Fn(&mut dyn store::Store, Data) -> Result<store::Id, error::Error>
//         + 'static
//         + Send
//         + Sync
//         + std::panic::RefUnwindSafe,
//     Data: store::Data + 'static + Send + Sync + std::panic::RefUnwindSafe,
// {
//     type Instance = Self;

//     fn new_handler(&self) -> gotham::anyhow::Result<Self::Instance> {
//         Ok(*self)
//     }
// }
