use proc_macro::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, FieldsNamed,
    FieldsUnnamed, Ident, Index, Type, Variant,
};

// This is **far** from simple or optimal, but allows us to automatically derive
// the `MapId` trait for most types. If there will be more cases, just add them
// here - it should be fine for the time being.
#[proc_macro_derive(MapId)]
pub fn derive_map_id(input: TokenStream) -> TokenStream {
    fn field_mapper(is_self: bool, ident: &impl ToTokens, ty: &Type) -> impl ToTokens {
        let target = is_self.then_some(quote! { self. });
        match format!("{}", quote! { #ty }).as_str() {
            "Id" => quote! { map(&#target #ident) },
            "BTreeSet < Id >" => quote! { #target #ident.iter().map(|x| map(x)).collect() },
            "Option < Id >" => quote! { #target #ident.as_ref().map(|x| map(x)) },
            "Vec < Id >" => quote! { #target #ident.iter().map(|x| map(x)).collect() },
            _ => quote! { #target #ident.map_id(map) },
        }
    }

    let DeriveInput {
        data,
        generics,
        ident: name,
        ..
    } = parse_macro_input!(input as DeriveInput);

    let handler = match data {
        Data::Enum(DataEnum { variants, .. }) => {
            let variants = variants.iter().map(|Variant { ident, fields, .. }| {
                let is_tuple = fields.iter().any(|field| field.ident.is_none());
                if is_tuple {
                    let (field_idents, field_mappers): (Vec<_>, Vec<_>) = fields
                        .iter()
                        .enumerate()
                        .map(|(index, Field { ty, .. })| {
                            let ident = Ident::new(&format!("_{index}"), Span::call_site().into());
                            let mapper = field_mapper(false, &ident, ty);
                            (ident, mapper)
                        })
                        .unzip();
                    quote! {
                        #name::#ident(#(#field_idents,)*) =>
                        #name::#ident(#(#field_mappers,)*)
                    }
                } else {
                    let (field_idents, field_mappers): (Vec<_>, Vec<_>) = fields
                        .iter()
                        .map(|Field { ident, ty, .. }| {
                            let mapper = field_mapper(false, ident, ty);
                            (ident, quote! { #ident: #mapper })
                        })
                        .unzip();
                    quote! {
                        #name::#ident { #(#field_idents,)* } =>
                        #name::#ident { #(#field_mappers,)* }
                    }
                }
            });

            quote! { match self { #(#variants,)* } }
        }
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => {
            let field_mappers = named.iter().map(|Field { ident, ty, .. }| {
                let mapper = field_mapper(true, ident, ty);
                quote! { #ident: #mapper }
            });
            quote! { #name { #(#field_mappers,)* } }
        }
        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { unnamed, .. }),
            ..
        }) => {
            let field_mappers = unnamed.iter().enumerate().map(|(index, Field { ty, .. })| {
                let ident = Index::from(index);
                field_mapper(true, &ident, ty)
            });
            quote! { #name(#(#field_mappers,)*) }
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
