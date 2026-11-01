//! Opt-in visibility for the existing white-box fixtures. The engine does not
//! depend on this crate in production builds; no source files are rewritten.
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Fields, ImplItem, Item, Visibility, parse_macro_input};

/// Applied only through `cfg_attr(feature = "test-support", ...)` in the engine.
/// Preserve trait implementations and function bodies, including local items.
#[proc_macro_attribute]
pub fn fixture(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as Item);
    expose(&mut item);
    item.into_token_stream().into()
}

fn fields(fields: &mut Fields) {
    for field in fields.iter_mut() {
        field.vis = Visibility::Public(Default::default());
    }
}

fn expose(item: &mut Item) {
    let public = Visibility::Public(Default::default());
    match item {
        Item::Struct(item) => {
            item.vis = public;
            fields(&mut item.fields);
        }
        Item::Union(item) => {
            item.vis = public;
            for field in &mut item.fields.named {
                field.vis = Visibility::Public(Default::default());
            }
        }
        Item::Enum(item) => item.vis = public,
        Item::Type(item) => item.vis = public,
        Item::Trait(item) => item.vis = public,
        Item::Fn(item) => item.vis = public,
        Item::Static(item) => item.vis = public,
        Item::Const(item) if item.ident != "_" => item.vis = public,
        Item::Use(item) if !matches!(item.vis, Visibility::Inherited) => item.vis = public,
        Item::Mod(item) => {
            item.vis = public;
            if let Some((_, content)) = &mut item.content {
                for child in content {
                    expose(child);
                }
            }
        }
        Item::Impl(item) if item.trait_.is_none() => {
            for child in &mut item.items {
                match child {
                    ImplItem::Fn(item) => item.vis = Visibility::Public(Default::default()),
                    ImplItem::Const(item) if item.ident != "_" => {
                        item.vis = Visibility::Public(Default::default())
                    }
                    ImplItem::Type(item) => item.vis = Visibility::Public(Default::default()),
                    _ => {}
                }
            }
        }
        Item::Macro(item)
            if item
                .mac
                .path
                .segments
                .last()
                .is_some_and(|s| s.ident == "thread_local") =>
        {
            let mut declarations: syn::File = syn::parse2(item.mac.tokens.clone())
                .expect("thread_local fixture declarations are Rust static items");
            for item in &mut declarations.items {
                expose(item);
            }
            item.mac.tokens = declarations.into_token_stream();
        }
        _ => {}
    }
}
