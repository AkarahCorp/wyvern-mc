mod actor;
mod message;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn actor(attr: TokenStream, item: TokenStream) -> TokenStream {
    actor::actor(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn message(attr: TokenStream, item: TokenStream) -> TokenStream {
    message::message(attr.into(), item.into()).into()
}
