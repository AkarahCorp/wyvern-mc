use core::panic;

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

impl std::fmt::Debug for MessageVariant {
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

        let Some(variant_attr) = function.attrs.first() else {
            return quote! { compile_error!("All function items must have a #[Variant] attribute"); };
        };
        let Meta::Path(path) = &variant_attr.meta else {
            return quote! { compile_error!("All function items must have a #[Variant] attribute"); };
        };
        if !path.segments.len() == 1 {
            return quote! { compile_error!("All function items must have a #[Variant] attribute"); };
        }

        let name = path.segments.get(0).unwrap().ident.clone();

        let parameters = function.sig.inputs.iter().cloned().collect::<Vec<_>>();
        let returns = function.sig.output.clone();

        function.attrs.remove(0);
        message_variants.push(MessageVariant {
            enum_name: attr.message_type.clone(),
            name,
            parameters,
            returns,
            base_function: function.clone(),
        });
    }

    let assoc_fns = message_variants.iter().map(|v| &v.base_function);
    let mapped_fns = message_variants.iter().map(create_fn_from_variant);
    let enum_types = message_variants
        .iter()
        .flat_map(create_enum_types_from_variant)
        .collect::<Vec<_>>();

    let enum_arms = message_variants
        .iter()
        .map(create_match_arm_from_variant)
        .collect::<Vec<_>>();

    let attr_actor_type = attr.actor_type;
    let attr_message_type = attr.message_type;

    let o = quote! {
        pub(crate) enum #attr_message_type {
            #(#enum_types)*
        }

        impl crate::actors::Actor for #target_type {
            fn handle_messages(&mut self) {
                for _ in 0..512 {
                    match self.receiver.try_recv() {
                        Ok(v) => {
                            match v {
                                #(#enum_arms)*
                            }
                        },
                        Err(flume::TryRecvError::Empty) => { return; },
                        Err(flume::TryRecvError::Disconnected) => { return; }
                    }
                }
            }
        }

        impl #target_type {
            #(#assoc_fns)*

            fn as_actor(&self) -> #attr_actor_type {
                #attr_actor_type { sender: self.sender.downgrade() }
            }
        }

        impl #attr_actor_type {
            #(#mapped_fns)*
        }
    };

    // std::fs::write(
    //     Path::new(&format!(
    //         "./target/macros/message/{}",
    //         target_type.to_token_stream().to_string().replace(" ", "")
    //     )),
    //     RustFmt::new().format_str(o.to_string()).unwrap(),
    // )
    // .unwrap();
    // eprintln!("o: {}", RustFmt::new().format_str(o.to_string()).unwrap());
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
        .filter_map(|x| match x {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
        .map(|x| *x.ty.clone())
        .collect::<Vec<_>>();

    let sender_type: Type = syn::parse2(quote! { flume::Sender<#rt> }).unwrap();
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
        .filter_map(|x| match x {
            FnArg::Receiver(_receiver) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
        .map(|x| *x.ty.clone())
        .collect::<Vec<_>>();

    let param_types = param_types.iter();

    let param_names: Vec<Ident> = variant
        .parameters
        .iter()
        .filter_map(|x| match x {
            FnArg::Receiver(_receiver) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
        .map(|x| *x.pat.clone())
        .map(|x| match x {
            syn::Pat::Ident(pat_ident) => pat_ident,
            _ => panic!("all patterns must be identifiers"),
        })
        .map(|x| x.ident)
        .collect::<Vec<_>>();

    let doc_attr_opt = variant.base_function.attrs.first();
    let doc_attr = doc_attr_opt.iter();

    let fn_vis = &variant.base_function.vis;

    let enum_type = variant.enum_name.clone();
    let enum_variant = variant.name.clone();

    let r = quote! {
        #(#doc_attr)*
        #fn_vis fn #name(&self, #(#param_names: #param_types),*) -> #rt {
            let Some(sender) = self.sender.upgrade() else {
                return Err(ActorError::ActorHasBeenDropped);
            };
            let (tx, mut rx) = flume::bounded(1);
            match sender.try_send(#enum_type::#enum_variant(#(#param_names,)* tx)) {
                Ok(v) => {
                    drop(sender);
                },
                Err(flume::TrySendError::Disconnected(v)) => {
                    return Err(ActorError::ActorHasBeenDropped);
                }
                Err(e) => {
                    panic!("something went wrong :(");
                }
            }
            loop {
                match rx.try_recv() {
                    Ok(v) => return v,
                    Err(e) if e == flume::TryRecvError::Disconnected => {
                        return Err(ActorError::ActorHasBeenDropped);
                    }
                    Err(e) => {
                        std::thread::yield_now();
                    }
                };
            };
        }
    };
    r
}

fn create_match_arm_from_variant(variant: &MessageVariant) -> TokenStream {
    let name = &variant.base_function.sig.ident;

    let param_names: Vec<Ident> = variant
        .parameters
        .iter()
        .filter_map(|x| match x {
            FnArg::Receiver(_receiver) => None,
            FnArg::Typed(pat_type) => Some(pat_type),
        })
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
            let r = self.#name(#(#param_names,)*);
            let _ = tx.try_send(r);
        }
    };
    r
}
