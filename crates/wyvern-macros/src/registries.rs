use proc_macro2::TokenStream;
use quote::quote;
use voxidian_protocol::value::{AttributeType, BlockState, EntityType, Identifier, SoundEvent};

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
        pub const #fn_name_ident: Id = Id::constant(#namespace, #path);
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
        pub const #fn_name_ident: Id = Id::constant(#namespace, #path);
    }
}

pub fn sounds(_item: TokenStream) -> TokenStream {
    let mut sounds: Vec<Identifier> = SoundEvent::vanilla_registry()
        .keys()
        .cloned()
        .map(|x| Identifier::new(x.namespace.to_lowercase(), x.path.to_lowercase()))
        .collect::<Vec<_>>();
    sounds.dedup();
    let blocks = sounds.iter().map(sound_to_tokens);

    quote! {
        impl Sounds {
            #(#blocks)*
        }
    }
}

fn sound_to_tokens(id: &Identifier) -> TokenStream {
    let path = &id.path;
    let namespace = &id.namespace;
    let fn_name_ident = proc_macro2::Ident::new(
        &path.to_uppercase().replace(".", "_"),
        proc_macro2::Span::call_site(),
    );

    quote! {
        pub const #fn_name_ident: Sound = Sound { name: Id::constant(#namespace, #path), pitch: 1.0, volume: 1.0, category: SoundCategory::Master };
    }
}

pub fn attrs(_item: TokenStream) -> TokenStream {
    let mut attrs: Vec<Identifier> = AttributeType::vanilla_registry()
        .keys()
        .cloned()
        .map(|x| Identifier::new(x.namespace.to_lowercase(), x.path.to_lowercase()))
        .collect::<Vec<_>>();
    attrs.dedup();
    let blocks = attrs.iter().map(attr_to_tokens);

    quote! {
        impl Attributes {
            #(#blocks)*
        }
    }
}

fn attr_to_tokens(id: &Identifier) -> TokenStream {
    let path = &id.path;
    let namespace = &id.namespace;
    let fn_name_ident = proc_macro2::Ident::new(
        &path.to_uppercase().replace(".", "_"),
        proc_macro2::Span::call_site(),
    );

    quote! {
        pub const #fn_name_ident: DataComponentType<f64> = DataComponentType::new(Id::constant(#namespace, #path));
    }
}
