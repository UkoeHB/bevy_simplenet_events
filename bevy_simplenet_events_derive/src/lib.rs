//module tree
mod events;

//proc exports
use proc_macro::TokenStream;

//-------------------------------------------------------------------------------------------------------------------

#[proc_macro_derive(SimplenetEvent)]
pub fn derive_simplenet_event(input: TokenStream) -> TokenStream
{
    events::derive_simplenet_event_impl(input)
}

//-------------------------------------------------------------------------------------------------------------------
