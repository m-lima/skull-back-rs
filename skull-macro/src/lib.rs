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
