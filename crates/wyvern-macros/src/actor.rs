use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemStruct, Type, parse::Parse, token::Comma};

#[derive(Debug, Clone)]
pub struct ActorInput {
    pub actor_type: Type,
    pub message_type: Type,
}

impl Parse for ActorInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let actor_type = Type::parse(input)?;
        let _ = Comma::parse(input)?;
        let message_type = Type::parse(input)?;
        Ok(ActorInput {
            actor_type,
            message_type,
        })
    }
}

pub fn actor(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: ActorInput = match syn::parse2(attr) {
        Ok(attr) => attr,
        Err(e) => return e.into_compile_error(),
    };

    let strct: ItemStruct = match syn::parse2(item.clone()) {
        Ok(strct) => strct,
        Err(e) => return e.into_compile_error(),
    };
    let strct_type = strct.ident;
    let fields = strct.fields.iter().collect::<Vec<_>>();

    let attr_actor_type = attr.actor_type;
    let attr_message_type = attr.message_type;

    let attributes = strct.attrs;

    let o = quote! {
        #[derive(Clone, Debug)]
        #(#attributes)*
        pub struct #attr_actor_type {
            pub(crate) sender: flume::WeakSender<#attr_message_type>
        }

        #(#attributes)*
        pub(crate) struct #strct_type {
            #(#fields),*,
            pub(crate) receiver: flume::Receiver<#attr_message_type>
        }
    };
    // std::fs::write(
    //     Path::new(&format!("./target/macros/actor/{}", strct_type)),
    //     RustFmt::new().format_str(o.to_string()).unwrap(),
    // )
    // .unwrap();
    // eprintln!("o: {}", RustFmt::new().format_str(o.to_string()).unwrap());
    o
}
