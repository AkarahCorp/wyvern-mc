use proc_macro2::TokenStream;
use quote::quote;
use voxidian_protocol::value::{BlockState, EntityType, Identifier};

pub fn entities(_item: TokenStream) -> TokenStream {
    let entities: Vec<TokenStream> = EntityType::vanilla_registry()
        .entries()
        .map(|x| x.1)
        .map(entity_type_to_tokens)
        .collect::<Vec<_>>();
    let entity_iter = entities.iter();
    quote! {
        impl Entities {
            #(#entity_iter)*
        }
    }
}

fn entity_type_to_tokens(entity: &EntityType) -> TokenStream {
    let path = &entity.id.path;
    let namespace = &entity.id.namespace;
    let fn_name_ident =
        proc_macro2::Ident::new(&path.to_uppercase(), proc_macro2::Span::call_site());

    quote! {
        pub const #fn_name_ident: Key<EntityType> = Key::constant(#namespace, #path);
    }
}

pub fn blocks(_item: TokenStream) -> TokenStream {
    let mut blocks: Vec<Identifier> = BlockState::all_block_states()
        .iter()
        .map(|x| x.id.clone())
        .collect::<Vec<_>>();
    blocks.dedup();
    let blocks = blocks.iter().map(block_to_tokens);

    quote! {
        impl Blocks {
            #(#blocks)*
        }
    }
}

fn block_to_tokens(id: &Identifier) -> TokenStream {
    let path = &id.path;
    let namespace = &id.namespace;
    let fn_name_ident =
        proc_macro2::Ident::new(&path.to_uppercase(), proc_macro2::Span::call_site());

    quote! {
        pub const #fn_name_ident: Key<Block> = Key::constant(#namespace, #path);
    }
}
