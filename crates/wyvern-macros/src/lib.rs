mod actor;
mod message;
mod registries;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn actor(attr: TokenStream, item: TokenStream) -> TokenStream {
    actor::actor(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn message(attr: TokenStream, item: TokenStream) -> TokenStream {
    message::message(attr.into(), item.into()).into()
}

#[proc_macro]
pub fn generate_entity_types(input: TokenStream) -> TokenStream {
    registries::entities(input.into()).into()
}

#[proc_macro]
pub fn generate_blocks_types(input: TokenStream) -> TokenStream {
    registries::blocks(input.into()).into()
}

#[proc_macro]
pub fn generate_sounds_types(input: TokenStream) -> TokenStream {
    registries::sounds(input.into()).into()
}
