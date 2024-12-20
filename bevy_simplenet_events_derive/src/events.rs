use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, DeriveInput};

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn derive_simplenet_event_impl(input: TokenStream) -> TokenStream
{
    let mut ast = parse_macro_input!(input as DeriveInput);
    ast.generics
        .make_where_clause()
        .predicates
        .push(parse_quote! { Self: Send + Sync + 'static });
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let struct_name = &ast.ident;

    TokenStream::from(quote! {
        impl #impl_generics ::bevy_simplenet_events::SimplenetEvent for #struct_name #ty_generics #where_clause {}
    })
}

//-------------------------------------------------------------------------------------------------------------------
