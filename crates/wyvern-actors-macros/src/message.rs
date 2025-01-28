use core::panic;
use std::fmt::Debug;

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

impl Debug for MessageVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageVariant")
            .field("enum_name", &self.enum_name.to_token_stream().to_string())
            .field("name", &self.name.to_token_stream().to_string())
            .field(
                "parameters",
                &self
                    .parameters
                    .iter()
                    .map(|x| x.to_token_stream().to_string())
                    .collect::<Vec<_>>(),
            )
            .field("returns", &self.returns.to_token_stream().to_string())
            .field(
                "base_function",
                &self.base_function.to_token_stream().to_string(),
            )
            .finish()
    }
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
    let enum_types = message_variants
        .iter()
        .map(|x| create_enum_types_from_variant(x))
        .flatten()
        .collect::<Vec<_>>();

    let enum_arms = message_variants
        .iter()
        .map(|x| create_match_arm_from_variant(x))
        .collect::<Vec<_>>();

    let attr_actor_type = attr.actor_type;
    let attr_message_type = attr.message_type;

    let o = quote! {

        pub(crate) enum #attr_message_type {
            #(#enum_types)*
        }
        impl wyvern_actors::Actor for #target_type {
            async fn handle_messages(&mut self) {
                loop {
                    match self.receiver.try_recv() {
                        Ok(v) => {
                            match v {
                                #(#enum_arms)*
                            }
                        },
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => { return; },
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => { return; }
                    }
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

    o
}

fn create_enum_types_from_variant(variant: &MessageVariant) -> TokenStream {
    let rt = match &variant.returns {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_rarrow, ty) => ty.to_token_stream(),
    };

    let mut param_types: Vec<Type> = variant
        .parameters
        .iter()
        .map(|x| match x {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .map(|x| *x.ty.clone())
        .collect::<Vec<_>>();

    let sender_type: Type = syn::parse2(quote! { tokio::sync::oneshot::Sender<#rt> }).unwrap();
    param_types.push(sender_type);

    let variant_name = &variant.name;

    let o = quote! {
        #variant_name ( #(#param_types,)* ),
    };
    o
}

fn create_fn_from_variant(variant: &MessageVariant) -> TokenStream {
    let name = &variant.base_function.sig.ident;
    let rt = match &variant.returns {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_rarrow, ty) => ty.to_token_stream(),
    };

    let param_types: Vec<Type> = variant
        .parameters
        .iter()
        .map(|x| match x {
            FnArg::Receiver(_receiver) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .map(|x| *x.ty.clone())
        .collect::<Vec<_>>();

    let param_types = param_types.iter();

    let param_names: Vec<Ident> = variant
        .parameters
        .iter()
        .map(|x| match x {
            FnArg::Receiver(_receiver) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .map(|x| *x.pat.clone())
        .map(|x| match x {
            syn::Pat::Ident(pat_ident) => pat_ident,
            _ => panic!("all patterns must be identifiers"),
        })
        .map(|x| x.ident)
        .collect::<Vec<_>>();

    let enum_type = variant.enum_name.clone();
    let enum_variant = variant.name.clone();

    let r = quote! {
        pub async fn #name(&self, #(#param_names: #param_types),*) -> #rt {
            let (tx, mut rx) = tokio::sync::oneshot::channel();
            self.sender.send(#enum_type::#enum_variant(#(#param_names,)* tx)).await.unwrap();
            rx.await.unwrap()
        }
    };
    r
}

fn create_match_arm_from_variant(variant: &MessageVariant) -> TokenStream {
    let name = &variant.base_function.sig.ident;

    let param_names: Vec<Ident> = variant
        .parameters
        .iter()
        .map(|x| match x {
            FnArg::Receiver(_receiver) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .map(|x| *x.pat.clone())
        .map(|x| match x {
            syn::Pat::Ident(pat_ident) => pat_ident,
            _ => panic!("all patterns must be identifiers"),
        })
        .map(|x| x.ident)
        .collect::<Vec<_>>();

    let enum_type = variant.enum_name.clone();
    let enum_variant = variant.name.clone();

    let r = quote! {
        #enum_type::#enum_variant(#(#param_names,)* tx) => {
            tx.send(self.#name(#(#param_names,)*).await).unwrap()
        }
    };
    r
}
