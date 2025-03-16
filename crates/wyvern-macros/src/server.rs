use proc_macro2::TokenStream;
use syn::{ItemFn, parse2};

pub fn server(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse2::<ItemFn>(item).unwrap();
    let name = func.sig.ident.clone();
    quote::quote! {
        fn main() {
            #func

            #name().run();
        }
    }
}
