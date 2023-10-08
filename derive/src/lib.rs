extern crate proc_macro;

#[proc_macro_derive(Data)]
pub fn derive_data(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_data_impl(syn::parse_macro_input!(input as syn::DeriveInput)).into()
}

fn compile_error(span: &impl syn::spanned::Spanned, message: &str) -> proc_macro2::TokenStream {
    syn::Error::new(span.span(), message).to_compile_error()
}

#[allow(clippy::needless_pass_by_value)]
fn derive_data_impl(input: syn::DeriveInput) -> proc_macro2::TokenStream {
    let visibility = input.vis.clone();
    let original_name = input.ident.clone();
    let with_id_name = quote::format_ident!("{}Id", original_name);

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let syn::Data::Struct(data) = input.data.clone() else {
        return compile_error(&input, "Data can only be derived for structs");
    };
    let fields = match data.fields {
        syn::Fields::Named(fields) => fields,
        syn::Fields::Unnamed(fields) => {
            return compile_error(
                &fields,
                "Data cannot be used with structs with unnamed fields",
            )
        }
        syn::Fields::Unit => return compile_error(&input, "Data cannot be used with unit structs"),
    };

    let fields = fields.named;
    let field_names = fields
        .iter()
        .map(|p| p.ident.as_ref().expect("Already verified to be named"))
        .collect::<Vec<_>>();

    quote::quote! {
        #[derive(Clone, Debug, PartialEq, ::serde::Serialize)]
        #visibility struct #with_id_name #impl_generics #where_clause {
            pub id: Id,
            #fields
        }

        impl #impl_generics WithId<#original_name #type_generics> for #with_id_name #type_generics #where_clause {
            fn new(id: Id, data: #original_name #type_generics) -> Self {
                Self {
                    id,
                    #(#field_names: data.#field_names),*
                }
            }

            fn forget_id(self) -> #original_name #type_generics {
                #original_name {
                    #(#field_names: self.#field_names),*
                }
            }

            fn id(&self) -> Id {
                self.id
            }
        }

        impl #impl_generics std::cmp::PartialEq<#original_name #type_generics> for #with_id_name #type_generics #where_clause {
            fn eq(&self, other: &#original_name #type_generics) -> bool {
                #(self.#field_names == other.#field_names)&&*
            }
        }

        impl #impl_generics Data for #original_name #type_generics #where_clause {
            type Id = #with_id_name #type_generics;
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn simple_case() {
        let input = quote::quote! {
            pub(crate) struct S {
                string: String,
                pub int: i32,
                pub(super) optional: Optional<bool>,
            }
        };

        let expected = quote::quote! {
            #[derive(Clone, Debug, PartialEq, ::serde::Serialize)]
            pub(crate) struct SId {
                pub id: Id,
                string: String,
                pub int: i32,
                pub(super) optional: Optional<bool>,
            }

            impl WithId<S> for SId {
                fn new(id: Id, data: S) -> Self {
                    Self {
                        id,
                        string: data.string,
                        int: data.int,
                        optional: data.optional
                    }
                }

                fn forget_id(self) -> S {
                    S {
                        string: self.string,
                        int: self.int,
                        optional: self.optional
                    }
                }

                fn id(&self) -> Id {
                    self.id
                }
            }

            impl std::cmp::PartialEq<S> for SId {
                fn  eq(&self, other: &S) -> bool {
                    self.string == other.string &&
                    self.int == other.int &&
                    self.optional == other.optional
                }
            }

            impl Data for S {
                type Id = SId;
            }
        }
        .to_string();

        assert_eq!(
            super::derive_data_impl(syn::parse2::<syn::DeriveInput>(input).unwrap()).to_string(),
            expected
        );
    }

    #[test]
    fn generics() {
        let input = quote::quote! {
            #[derive(Debug)]
            struct S<'a, A, B: Bee, C, const L: usize>
            where C: Cee
            {
                a: &'a A,
                b: B,
                c: C,
                l: [u8; L],
            }
        };

        let expected = quote::quote! {
            #[derive(Clone, Debug, PartialEq, ::serde::Serialize)]
            struct SId<'a, A, B: Bee, C, const L: usize>
            where C: Cee
            {
                pub id: Id,
                a: &'a A,
                b: B,
                c: C,
                l: [u8; L],
            }

            impl<'a, A, B: Bee, C, const L: usize> WithId<S<'a, A, B, C, L> > for SId<'a, A, B, C, L>
            where C: Cee
            {
                fn new(id: Id, data: S<'a, A, B, C, L>) -> Self {
                    Self {
                        id,
                        a: data.a,
                        b: data.b,
                        c: data.c,
                        l: data.l
                    }
                }

                fn forget_id(self) -> S<'a, A, B, C, L> {
                    S {
                        a: self.a,
                        b: self.b,
                        c: self.c,
                        l: self.l
                    }
                }

                fn id(&self) -> Id {
                    self.id
                }
            }

            impl<'a, A, B: Bee, C, const L: usize> std::cmp::PartialEq<S<'a, A, B, C, L> > for SId<'a, A, B, C, L>
            where C: Cee
            {
                fn  eq(&self, other: &S<'a, A, B, C, L> ) -> bool {
                    self.a == other.a &&
                    self.b == other.b &&
                    self.c == other.c &&
                    self.l == other.l
                }
            }

            impl<'a, A, B: Bee, C, const L: usize> Data for S<'a, A, B, C, L>
            where C: Cee
            {
                type Id = SId<'a, A, B, C, L>;
            }
        }
        .to_string();

        assert_eq!(
            super::derive_data_impl(syn::parse2::<syn::DeriveInput>(input).unwrap()).to_string(),
            expected
        );
    }

    #[test]
    fn error() {
        let input = quote::quote! {
            struct S;
        };
        assert_eq!(
            super::derive_data_impl(syn::parse2::<syn::DeriveInput>(input).unwrap()).to_string(),
            ":: core :: compile_error ! { \"Data cannot be used with unit structs\" }"
        );

        let input = quote::quote! {
            struct S(String);
        };
        assert_eq!(
            super::derive_data_impl(syn::parse2::<syn::DeriveInput>(input).unwrap()).to_string(),
            ":: core :: compile_error ! { \"Data cannot be used with structs with unnamed fields\" }"
        );

        let input = quote::quote! {
            enum S{String}
        };
        assert_eq!(
            super::derive_data_impl(syn::parse2::<syn::DeriveInput>(input).unwrap()).to_string(),
            ":: core :: compile_error ! { \"Data can only be derived for structs\" }"
        );

        let input = quote::quote! {
            union S{
                a: u32,
                b: f32,
            }
        };
        assert_eq!(
            super::derive_data_impl(syn::parse2::<syn::DeriveInput>(input).unwrap()).to_string(),
            ":: core :: compile_error ! { \"Data can only be derived for structs\" }"
        );
    }
}
