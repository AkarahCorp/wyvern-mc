use core::panic;
use std::env::var;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Meta, ReturnType, Type};

use crate::actor::ActorInput;

struct MessageVariant {
    enum_name: Type,
    name: Ident,
    parameters: Vec<FnArg>,
    returns: ReturnType,
    base_function: ImplItemFn,
}
pub fn message(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr: ActorInput = match syn::parse2(attr) {
        Ok(message_type) => message_type,
        Err(e) => return e.to_compile_error(),
    };

    // TODO: autogenerate message enum

    let mut impl_block: ItemImpl = match syn::parse2(item.clone()) {
        Ok(impl_block) => impl_block,
        Err(e) => return e.to_compile_error(),
    };

    let target_type = impl_block.self_ty.clone();

    let mut message_variants: Vec<MessageVariant> = Vec::new();
    for element in &mut impl_block.items {
        let ImplItem::Fn(function) = element else {
            return quote! { compile_error!("Expected only function items in `impl` block"); };
        };

        let Some(variant_attr) = function.attrs.get(0) else {
            return quote! { compile_error!("All function items must have a #[Variant] attribute"); };
        };
        let Meta::Path(path) = &variant_attr.meta else {
            return quote! { compile_error!("All function items must have a #[Variant] attribute"); };
        };
        if !path.segments.len() == 1 {
            return quote! { compile_error!("All function items must have a #[Variant] attribute"); };
        }
        let name = path.segments.get(0).unwrap().ident.clone();

        let parameters = function
            .sig
            .inputs
            .iter()
            .map(|x| x.clone())
            .collect::<Vec<_>>();
        let returns = function.sig.output.clone();

        function.attrs.clear();
        message_variants.push(MessageVariant {
            enum_name: attr.message_type.clone(),
            name,
            parameters,
            returns,
            base_function: function.clone(),
        });
    }

    let assoc_fns = message_variants.iter().map(|v| &v.base_function);
    let mapped_fns = message_variants.iter().map(|x| create_fn_from_variant(x));

    let attr_actor_type = attr.actor_type;

    let o = quote! {
        impl wyvern_mc::actors::Actor for #target_type {
            async fn handle_messages(self) {
                loop {
                    eprintln!("I'm matching!");
                    // TODO: actually match on messages
                    tokio::task::yield_now().await;
                }
            }
        }

        impl #target_type {
            #(#assoc_fns)*
        }

        impl #attr_actor_type {
            #(#mapped_fns)*
        }
    };
    eprintln!("\n\n{}\n\n", o.to_string());
    o
}

fn create_fn_from_variant(variant: &MessageVariant) -> TokenStream {
    let name = &variant.base_function.sig.ident;
    let rt = match &variant.returns {
        ReturnType::Default => quote! { () },
        ReturnType::Type(rarrow, ty) => ty.to_token_stream(),
    };

    let mut param_types: Vec<Type> = variant
        .parameters
        .iter()
        .map(|x| match x {
            FnArg::Receiver(receiver) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .map(|x| *x.ty.clone())
        .collect::<Vec<_>>();

    let mut enum_types = param_types.clone();
    match &variant.returns {
        ReturnType::Default => {}
        ReturnType::Type(_, ty) => enum_types.push(*ty.clone()),
    }
    let enum_types = enum_types.into_iter().map(|x| x);
    let param_types = param_types.into_iter().map(|x| x);

    let enum_type = variant.enum_name.clone();
    let enum_variant = variant.name.clone();

    let base_name = variant.base_function.sig.ident.clone();

    // TODO: add support for input parameters in the enum variant
    let r = quote! {
        pub async fn #name(&self, #(#param_types),*) -> #rt {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.sender.send(#enum_type::#enum_variant(tx)).await;
            rx.await.unwrap()
        }
    };
    r
}
