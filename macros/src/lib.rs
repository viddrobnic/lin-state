use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DeriveInput, GenericParam, Generics,
    Index,
};

#[proc_macro_derive(Resource)]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    // Add a bound `T: Resource` to every type parameter T.
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate an expression to be used in clone_state method and
    // an expression to be used in set_cleanup_enabled method.
    let cloned_state = clone_state_expr(&input.data);
    let cleanup_enabled = set_cleanup_enabled_expr(&input.data);

    let expanded = quote! {
        // The generated impl.
        impl #impl_generics guard::resource::Resource for #name #ty_generics #where_clause {
            unsafe fn clone_state(&self) -> Self {
                #cloned_state
            }

            unsafe fn set_cleanup_enabled(&mut self, cleanup_enabled: bool) {
                #cleanup_enabled
            }
        }
    };

    TokenStream::from(expanded)
}

// Add a bound `T: Resource` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(guard::resource::Resource));
        }
    }
    generics
}

// Generate an expression to clone the resource state.
fn clone_state_expr(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                // Expand to expression like:
                // ```
                // Self { a: self.a.clone_state(), b: self.b.clone_state(), }
                // ```
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned!(f.span() => #name: self.#name.clone_state())
                });

                quote!(Self { #(#recurse),* })
            }
            syn::Fields::Unnamed(ref fields) => {
                // Expand to expression like:
                // ```
                // Self(self.0.clone_state(), self.1.clone_state())
                // ```
                let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let index = Index::from(i);
                    quote_spanned!(f.span() => self.#index.clone_state())
                });

                quote!(Self(#(#recurse),*))
            }
            // Unit structs cannot implement the Resource trait, since we can not change if cleanup
            // is enabled. To change if cleanup is enabled, at least one field must be present in the struct.
            syn::Fields::Unit => panic!("Unit structs are not supported"),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}

// Generate an expression to set the cleanup_enabled flag.
fn set_cleanup_enabled_expr(data: &Data) -> proc_macro2::TokenStream {
    match *data {
        Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                // Expand to expression like:
                // ```
                // self.a.set_cleanup_enabled(cleanup_enabled);
                // self.b.set_cleanup_enabled(cleanup_enabled);
                // ```
                let recourse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned!(f.span() => self.#name.set_cleanup_enabled(cleanup_enabled);)
                });

                quote!(#(#recourse)*)
            }
            syn::Fields::Unnamed(ref fields) => {
                // Expand to expression like:
                // ```
                // self.0.set_cleanup_enabled(cleanup_enabled);
                // self.1.set_cleanup_enabled(cleanup_enabled);
                // ```
                let recourse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let index = Index::from(i);
                    quote_spanned!(f.span() => self.#index.set_cleanup_enabled(cleanup_enabled);)
                });

                quote!(#(#recourse)*)
            }
            // Unit structs cannot implement the Resource trait, since we can not change if cleanup
            // is enabled. To change if cleanup is enabled, at least one field must be present in the struct.
            syn::Fields::Unit => panic!("Unit structs are not supported"),
        },
        Data::Enum(_) | Data::Union(_) => unimplemented!(),
    }
}
