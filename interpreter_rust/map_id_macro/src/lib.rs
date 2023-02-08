use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Variant,
};

// This is **far** from simple or optimal, but allows us to automatically derive
// the `MapId` trait for most types. If there will be more cases, just add them
// here - it should be fine for the time being.
#[proc_macro_derive(MapId)]
pub fn derive_map_id(input: TokenStream) -> TokenStream {
    let DeriveInput {
        data,
        generics,
        ident: name,
        ..
    } = parse_macro_input!(input as DeriveInput);

    fn field_mapper(target: impl ToTokens, Field { ident, ty, .. }: &Field) -> impl ToTokens {
        match format!("{}", quote! { #ty }).as_str() {
            "bool" => quote! { #ident: *#target #ident },
            "Id" => quote! { #ident: map(&#target #ident) },
            "Vec < Id >" => quote! { #ident: #target #ident.iter().map(map).collect() },
            _ => quote! { #ident: #target #ident.map_id(map) },
        }
    }

    let handler = match data {
        Data::Enum(DataEnum { variants, .. }) => {
            let variants = variants.iter().map(|Variant { ident, fields, .. }| {
                let field_idents = fields.iter().map(|Field { ident, .. }| quote! { #ident });
                let field_mappers = fields.iter().map(|field| field_mapper(quote! {}, field));
                quote! {
                    #name::#ident { #(#field_idents,)* } =>
                    #name::#ident { #(#field_mappers,)* }
                }
            });

            quote! { match self { #(#variants,)* } }
        }
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => {
            let field_mappers = named
                .iter()
                .map(|field| field_mapper(quote! { self. }, field));
            quote! { #name { #(#field_mappers,)* } }
        }
        _ => unimplemented!(),
    };

    let boundary = generics.params.iter().find_map(|generic| {
        match format!("{}", quote! { #generic }).as_str() {
            "Id" => Some(quote! {}),
            "Id : Ord" => Some(quote! { : Ord }),
            _ => unimplemented!(),
        }
    });

    let implementation = quote! {
        impl<OldId #boundary, NewId #boundary> MapId<#name<NewId>, OldId, NewId> for #name<OldId> {
            fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> #name<NewId> {
                #handler
            }
        }
    };

    TokenStream::from(implementation)
}
