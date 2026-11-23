//! `#[derive(TokenSpec)]` for the token structs of `dodai-core`.
//!
//! ```ignore
//! #[derive(TokenSpec)]
//! #[token_spec(resolver = "tag", args(tone: Tone, solid: bool), doc = "Tag / badge.")]
//! pub struct TagTokens {
//!     pub background: Option<Color>,
//!     pub foreground: Color,
//! }
//! ```
//!
//! It implements `TokenSpec` (the component's resolver, arguments and colour fields, which the
//! `dodai-gen` tool turns into per-toolkit glue) and makes the compiler check the declaration:
//! a hidden function calls `Theme::<resolver>(..)` with the declared argument types and
//! requires it to return the struct, and a field that is neither `Color` nor `Option<Color>`
//! is an error. A renamed resolver, a changed signature or a new field kind breaks the build
//! here instead of in a generated file.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as Tokens;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    spanned::Spanned,
    Data, DeriveInput, Fields, GenericArgument, Ident, LitStr, PathArguments, Token, Type,
};

/// `name: Type` in `args(..)`.
struct ArgDecl {
    name: Ident,
    ty: Type,
}

impl Parse for ArgDecl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        Ok(Self {
            name,
            ty: input.parse()?,
        })
    }
}

/// The last path segment of `ty` when it is `name` or `name<inner>`.
fn segment(ty: &Type) -> Option<(&Ident, Option<&Type>)> {
    let Type::Path(p) = ty else { return None };
    let seg = p.path.segments.last()?;
    match &seg.arguments {
        PathArguments::None => Some((&seg.ident, None)),
        PathArguments::AngleBracketed(a) => match a.args.first()? {
            GenericArgument::Type(t) => Some((&seg.ident, Some(t))),
            _ => None,
        },
        PathArguments::Parenthesized(_) => None,
    }
}

/// `Color` or `Option<Color>`: the kind name of a field type.
fn kind(ty: &Type) -> Option<&'static str> {
    let is_color = |t: &Type| segment(t).is_some_and(|(i, a)| i == "Color" && a.is_none());
    if is_color(ty) {
        return Some("Color");
    }
    let (outer, inner) = segment(ty)?;
    (outer == "Option" && inner.is_some_and(is_color)).then_some("OptColor")
}

fn expand(input: DeriveInput) -> syn::Result<Tokens> {
    let ident = &input.ident;
    let mut resolver: Option<LitStr> = None;
    let mut doc = LitStr::new("", ident.span());
    let mut args: Vec<ArgDecl> = Vec::new();
    for attr in input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("token_spec"))
    {
        attr.parse_nested_meta(|m| {
            if m.path.is_ident("resolver") {
                resolver = Some(m.value()?.parse()?);
            } else if m.path.is_ident("doc") {
                doc = m.value()?.parse()?;
            } else if m.path.is_ident("args") {
                let inner;
                parenthesized!(inner in m.input);
                args = Punctuated::<ArgDecl, Token![,]>::parse_terminated(&inner)?
                    .into_iter()
                    .collect();
            } else {
                return Err(m.error("expected `resolver`, `doc` or `args`"));
            }
            Ok(())
        })?;
    }
    let resolver = resolver.ok_or_else(|| {
        syn::Error::new_spanned(ident, "missing #[token_spec(resolver = \"...\")]")
    })?;
    let method = Ident::new(&resolver.value(), resolver.span());

    let Data::Struct(data) = &input.data else {
        return Err(syn::Error::new_spanned(ident, "TokenSpec is for structs"));
    };
    let Fields::Named(named) = &data.fields else {
        return Err(syn::Error::new_spanned(
            ident,
            "TokenSpec needs named fields",
        ));
    };
    let mut fields = Vec::new();
    for f in &named.named {
        let name = f.ident.as_ref().expect("named").to_string();
        let k = kind(&f.ty).ok_or_else(|| {
            syn::Error::new_spanned(&f.ty, "token fields must be `Color` or `Option<Color>`")
        })?;
        let k = Ident::new(k, f.ty.span());
        fields.push(quote! { ::dodai_core::component_spec::Field {
            name: #name,
            kind: ::dodai_core::component_spec::FieldKind::#k,
        } });
    }

    let arg_names: Vec<_> = args.iter().map(|a| &a.name).collect();
    let arg_tys: Vec<_> = args.iter().map(|a| &a.ty).collect();
    let arg_meta = args.iter().map(|a| {
        let n = a.name.to_string();
        let t = &a.ty;
        let t = quote!(#t).to_string().replace(' ', "");
        quote! { ::dodai_core::component_spec::Arg { name: #n, ty: #t } }
    });

    Ok(quote! {
        impl ::dodai_core::component_spec::TokenSpec for #ident {
            const SPEC: ::dodai_core::component_spec::ComponentSpec =
                ::dodai_core::component_spec::ComponentSpec {
                    name: #resolver,
                    doc: #doc,
                    resolver: #resolver,
                    tokens: stringify!(#ident),
                    args: &[#(#arg_meta),*],
                    fields: &[#(#fields),*],
                };
        }
        // Never called: makes the compiler check the declared resolver and its signature.
        const _: fn() = || {
            let _resolve = |theme: &::dodai_core::Theme #(, #arg_names: #arg_tys)*| -> #ident {
                theme.#method(#(#arg_names),*)
            };
        };
    })
}

/// See the crate docs.
#[proc_macro_derive(TokenSpec, attributes(token_spec))]
pub fn token_spec(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput))
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
