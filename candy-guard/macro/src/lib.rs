use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(GuardSet)]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = ast.data
    {
        named
    } else {
        panic!("No fields found");
    };

    let is_option_t = |ty: &syn::Type| -> bool {
        if let syn::Type::Path(ref p) = ty {
            if p.path.segments.len() != 1 || p.path.segments[0].ident != "Option" {
                return false;
            }
            if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
                if inner_ty.args.len() != 1 {
                    return false;
                } else if let syn::GenericArgument::Type(ref _ty) = inner_ty.args.first().unwrap() {
                    return true;
                }
            }
        }
        false
    };

    let unwrap_option_t = |ty: &syn::Type| -> syn::Type {
        if let syn::Type::Path(ref p) = ty {
            if p.path.segments.len() != 1 || p.path.segments[0].ident != "Option" {
                panic!("Type was not Option<T>");
            }
            if let syn::PathArguments::AngleBracketed(ref inner_ty) = p.path.segments[0].arguments {
                if inner_ty.args.len() != 1 {
                    panic!("Option type was not Option<T>");
                } else if let syn::GenericArgument::Type(ref ty) = inner_ty.args.first().unwrap() {
                    return ty.clone();
                }
            }
        }
        panic!("Type was not Option<T>");
    };

    let from_data = fields.iter().map(|f| {
        let name = &f.ident;

        if is_option_t(&f.ty) {
            let ty = unwrap_option_t(&f.ty);
            quote! {
                let #name = if #ty::is_enabled(features) {
                    cursor += #ty::size();
                    #ty::load(data, cursor)?
                } else {
                    None
                };
            }
        } else {
            quote! {}
        }
    });

    let to_data = fields.iter().map(|f| {
        let name = &f.ident;

        if is_option_t(&f.ty) {
            let ty = unwrap_option_t(&f.ty);
            quote! {
                if let Some(#name) = &self.#name {
                    cursor += #ty::size();
                    if cursor <= data.len() {
                        #name.save(data, cursor - #ty::size())?;
                        features = #ty::enable(features);
                    } else {
                        return err!(crate::errors::CandyGuardError::InvalidAccountSize);
                    }
                }
            }
        } else {
            quote! {}
        }
    });

    let merge_data = fields.iter().map(|f| {
        let name = &f.ident;

        if is_option_t(&f.ty) {
            quote! {
                if let Some(#name) = other.#name {
                    self.#name = Some(#name);
                }
            }
        } else {
            quote! {}
        }
    });

    let struct_fields = fields.iter().map(|f| {
        let name = &f.ident;
        quote! { #name }
    });

    let enabled = fields.iter().map(|f| {
        let name = &f.ident;

        if is_option_t(&f.ty) {
            quote! {
                if let Some(#name) = &self.#name {
                    conditions.push(#name);
                }
            }
        } else {
            quote! {}
        }
    });

    let struct_size = fields.iter().map(|f| {
        let name = &f.ident;

        if is_option_t(&f.ty) {
            let ty = unwrap_option_t(&f.ty);
            quote! {
                if self.#name.is_some() {
                    size += #ty::size();
                }
            }
        } else {
            quote! {}
        }
    });

    let bytes_count = fields.iter().map(|f| {
        if is_option_t(&f.ty) {
            let ty = unwrap_option_t(&f.ty);
            quote! {
                if #ty::is_enabled(features) {
                    count += #ty::size();
                }
            }
        } else {
            quote! {}
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn from_data(data: &[u8]) -> anchor_lang::Result<(Self, u64)> {
                let mut cursor = 0;

                let features = u64::from_le_bytes(*arrayref::array_ref![data, cursor, 8]);
                cursor += 8;

                #(#from_data)*

                Ok((Self {
                    #(#struct_fields,)*
                }, features))
            }

            pub fn bytes_count(features: u64) -> usize {
                let mut count = 8; // features (u64)
                #(#bytes_count)*
                count
            }

            pub fn to_data(&self, data: &mut [u8]) -> anchor_lang::Result<u64> {
                let mut features = 0;
                // leave space to write the features flag at the end
                let mut cursor = 8;

                #(#to_data)*

                // features
                data[0..8].copy_from_slice(&u64::to_le_bytes(features));

                Ok(features)
            }

            pub fn merge(&mut self, other: GuardSet) {
                #(#merge_data)*
            }

            pub fn enabled_conditions(&self) -> Vec<&dyn Condition> {
                // list of condition trait objects
                let mut conditions: Vec<&dyn Condition> = vec![];
                #(#enabled)*

                conditions
            }

            pub fn size(&self) -> usize {
                let mut size = 8; // features (u64)
                #(#struct_size)*
                size
            }
        }
    };

    TokenStream::from(expanded)
}
