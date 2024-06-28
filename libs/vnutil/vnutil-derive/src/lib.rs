use quote::{quote_spanned, quote};
use syn::{parse_macro_input, DeriveInput, GenericParam, parse_quote, Data, Fields, spanned::Spanned, Index};



#[proc_macro_derive(WriteTo)]
pub fn derive_write_to(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data) = &input.data {
        let generics = {
            let mut generics = input.generics;
            for param in generics.params.iter_mut() {
                if let GenericParam::Type(type_param) = param {
                    type_param.bounds.push(parse_quote!(vnutil::io::WriteTo));
                }
            }
            generics
        };

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let stream = match &data.fields {
            Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() => vnutil::io::WriteTo::write_to(&self.#name, w)?; }
                });
                quote! {
                    #(
                        #recurse
                    )*
                }
            }
            Fields::Unnamed(fields) => {
                let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let index = Index::from(i);
                    quote_spanned! { f.span() => vnutil::io::WriteTo::write_to(&self.#index, w)?; }
                });

                quote! {
                    #(
                        #recurse
                    )*
                }
            }
            Fields::Unit => {
                quote!()
            }
        };

        let name = input.ident;

        let expanded = quote! {
            impl #impl_generics vnutil::io::WriteTo for #name #ty_generics #where_clause {
                fn write_to<W: ?Sized + std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
                    #stream
                    Ok(())
                }
            }
        };

        proc_macro::TokenStream::from(expanded)
    } else {
        proc_macro::TokenStream::new()
    }

}

#[proc_macro_derive(ReadFrom)]
pub fn derive_read_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data) = &input.data {
        let generics = {
            let mut generics = input.generics;
            for param in generics.params.iter_mut() {
                if let GenericParam::Type(type_param) = param {
                    type_param.bounds.push(parse_quote!(vnutil::io::ReadFrom));
                }
            }
            generics
        };

        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let stream = match &data.fields {
            Fields::Named(fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() => #name: vnutil::io::ReadFrom::read_from(r)?, }
                });
                quote! {
                    Self {#(
                        #recurse
                    )*}
                }
            }
            Fields::Unnamed(fields) => {
                let recurse = fields.unnamed.iter().enumerate().map(|(i, f)| {
                    let index = Index::from(i);
                    quote_spanned! { f.span() => #index: vnutil::io::ReadFrom::read_from(r)?, }
                });

                quote! {
                    Self {#(
                        #recurse
                    )*}
                }
            }
            Fields::Unit => {
                quote! {
                    Self
                }
            }
        };

        let name = input.ident;

        let expanded = quote! {
            impl #impl_generics vnutil::io::ReadFrom for #name #ty_generics #where_clause {
                fn read_from<R: ?Sized + std::io::Read>(r: &mut R) -> std::io::Result<Self> {
                    Ok(#stream)
                }
            }
        };

        proc_macro::TokenStream::from(expanded)
    } else {
        proc_macro::TokenStream::new()
    }

}