struct TypeParams<'a>(Vec<&'a syn::TypeParam>);

impl quote::ToTokens for TypeParams<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use quote::TokenStreamExt;

        for type_param in &self.0 {
            tokens.append(type_param.ident.clone());
            tokens.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Joint));
        }
    }
}

#[proc_macro_derive(HandlerCopy)]
pub fn handler_copy(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let syn::DeriveInput {
        ident, generics, ..
    } = syn::parse_macro_input!(input);

    let types = TypeParams(generics.type_params().collect());
    let handler_func = types.0[0].clone();

    let where_clause = generics
        .where_clause
        .clone()
        .expect("The handler function must be bound");

    let output = quote::quote! {
        impl<#types> Clone for #ident<#types>
            #where_clause,
            #handler_func: Copy,
        {
            fn clone(&self) -> Self {
                Self(self.0)
            }
        }

        impl<#types> Copy for #ident<#types>
            #where_clause,
            #handler_func: Copy,
        {
        }
    };

    output.into()
}

#[proc_macro_attribute]
pub fn handler(
    _attributes: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    create_handler(syn::parse_macro_input!(item))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn create_handler(handler: syn::ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    let signature = handler.sig;

    let inputs = &signature.inputs;
    if inputs.len() != 2 {
        return Err(syn::Error::new_spanned(
            inputs,
            "Expected two parameters: state and a handler function",
        ));
    }

    let generics = &signature.generics;

    if generics.gt_token.is_none() {
        return Err(syn::Error::new_spanned(
            signature,
            "Expected a generic handler function",
        ));
    }

    let where_clause = match generics.where_clause {
        Some(ref where_clause) => where_clause.clone(),
        None => {
            return Err(syn::Error::new_spanned(
                signature,
                "Expected the trait bounds to be in a where clause",
            ))
        }
    };

    let ident = signature.ident.clone();

    let a = handler.block;

    let output = quote::quote! {
        #[allow(non_camel_case_types)]
        pub struct #ident();
    };

    Ok(output)
}

struct HandlerFunc {
    asyncness: bool,
    unsafety: bool,
    ident: syn::Ident,
    generics: Generics,
    params: Params,
}

impl syn::parse::Parse for HandlerFunc {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.call(syn::Attribute::parse_outer)?;
        input.parse::<syn::Visibility>()?;
        input.parse::<Option<syn::Token!(const)>>()?;

        let asyncness = input.parse::<Option<syn::Token!(async)>>()?.is_some();
        let unsafety = input.parse::<Option<syn::Token!(unsafe)>>()?.is_some();

        input.parse::<Option<syn::Abi>>()?;
        input.parse::<syn::Token!(fn)>()?;
        let ident: syn::Ident = input.parse()?;

        Err(syn::Error::new_spanned(input, "nei!"))

        // let mut generics: Generics = input.parse()?;

        // let content;
        // let paren_token = parenthesized!(content in input);
        // let mut inputs = parse_fn_args(&content)?;
        // let variadic = pop_variadic(&mut inputs);

        // let output: ReturnType = input.parse()?;
        // generics.where_clause = input.parse()?;

        // Ok(Signature {
        //     constness,
        //     asyncness,
        //     unsafety,
        //     abi,
        //     fn_token,
        //     ident,
        //     generics,
        //     paren_token,
        //     inputs,
        //     variadic,
        //     output,
        // })
    }
}

struct Params {}

struct Generics {}

// impl<HandlerFunc, Output> gotham::handler::Handler for List<HandlerFunc, Output>
// where
//     HandlerFunc: Fn(&mut dyn store::Store) -> Result<Output, error::Error> + 'static + Send,
//     Output: serde::Serialize + 'static,
// {
//     fn handle(
//         self,
//         mut state: gotham::state::State,
//     ) -> std::pin::Pin<Box<gotham::handler::HandlerFuture>> {
//         Box::pin(async {
//             match self.handle(&mut state).await {
//                 Ok(r) => Ok((state, r)),
//                 Err(e) => Err((state, e.into_handler_error())),
//             }
//         })
//     }
// }

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
