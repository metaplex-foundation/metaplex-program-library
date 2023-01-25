use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{
    self, parse_macro_input, DeriveInput, Expr, ExprPath, GenericArgument, Lit, Meta, MetaList,
    MetaNameValue, NestedMeta, Path, PathArguments, Type, TypePath,
};

#[derive(Default)]
struct Variant {
    pub name: String,
    pub tuple: Option<String>,
    pub accounts: Vec<Account>,
    // (name, type, generic type)
    pub args: Vec<(String, String, Option<String>)>,
}

#[derive(Debug)]
struct Account {
    pub name: String,
    pub optional: bool,
}

// Helper account attribute (reusing from shank annotation).
const ACCOUNT_ATTRIBUTE: &str = "account";
// Helper args attribute.
const ARGS_ATTRIBUTE: &str = "args";
// Name property in the account attribute.
const NAME_PROPERTY: &str = "name";
// Optional property in the account attribute.
const OPTIONAL_PROPERTY: &str = "optional";

#[proc_macro_derive(AccountContext, attributes(account, args))]
pub fn account_context_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    // identifies the accounts associated with each enum variant

    let variants = if let syn::Data::Enum(syn::DataEnum { ref variants, .. }) = ast.data {
        let mut enum_variants = Vec::new();

        for v in variants {
            // extract the enum data (if there is one present)
            let mut variant = Variant {
                tuple: if let syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) = &v.fields {
                    match unnamed.first() {
                        Some(syn::Field {
                            ty:
                                Type::Path(TypePath {
                                    path: Path { segments, .. },
                                    ..
                                }),
                            ..
                        }) => Some(segments.first().unwrap().ident.to_string()),
                        _ => None,
                    }
                } else {
                    None
                },
                name: v.ident.to_string(),
                ..Default::default()
            };

            // parse the attribute of the variant
            for a in &v.attrs {
                let syn::Attribute {
                    path: syn::Path { segments, .. },
                    ..
                } = &a;
                let mut skip = true;
                let mut attribute = String::new();

                for path in segments {
                    let ident = path.ident.to_string();
                    // we are only interested in #[account] and #[args] attributes
                    if ident == ACCOUNT_ATTRIBUTE || ident == ARGS_ATTRIBUTE {
                        attribute = ident;
                        skip = false;
                    }
                }

                if !skip {
                    if attribute == ACCOUNT_ATTRIBUTE {
                        let meta_tokens = a.parse_meta().unwrap();
                        let nested_meta = if let Meta::List(MetaList { nested, .. }) = &meta_tokens
                        {
                            nested
                        } else {
                            panic!("#[account] requires attributes account name");
                        };

                        // (name, optional)
                        let mut property: (Option<String>, Option<String>) = (None, None);

                        for element in nested_meta {
                            match element {
                                // name = value (ignores any other attribute)
                                NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                                    path,
                                    lit,
                                    ..
                                })) => {
                                    let ident = path.get_ident();
                                    if let Some(ident) = ident {
                                        if *ident == NAME_PROPERTY {
                                            let token = match lit {
                                                // removes the surrounding "'s from string values"
                                                Lit::Str(lit) => {
                                                    lit.token().to_string().replace('\"', "")
                                                }
                                                _ => panic!("Invalid value for property {ident}"),
                                            };
                                            property.0 = Some(token);
                                        }
                                    }
                                }
                                // optional
                                NestedMeta::Meta(Meta::Path(path)) => {
                                    let name = path.get_ident().map(|x| x.to_string());
                                    if let Some(name) = name {
                                        if name == OPTIONAL_PROPERTY {
                                            property.1 = Some(name);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        variant.accounts.push(Account {
                            name: property.0.unwrap(),
                            optional: property.1.is_some(),
                        });
                    } else if attribute == ARGS_ATTRIBUTE {
                        let args_tokens: syn::ExprType = a.parse_args().unwrap();
                        // name
                        let name = match *args_tokens.expr {
                            Expr::Path(ExprPath {
                                path: Path { segments, .. },
                                ..
                            }) => segments.first().unwrap().ident.to_string(),
                            _ => panic!("#[args] requires an expression 'name: type'"),
                        };
                        // type
                        match *args_tokens.ty {
                            Type::Path(TypePath {
                                path: Path { segments, .. },
                                ..
                            }) => {
                                let segment = segments.first().unwrap();

                                // check whether we are dealing with a generic type
                                let generic_ty = match &segment.arguments {
                                    PathArguments::AngleBracketed(arguments) => {
                                        if let Some(GenericArgument::Type(Type::Path(ty))) =
                                            arguments.args.first()
                                        {
                                            Some(
                                                ty.path.segments.first().unwrap().ident.to_string(),
                                            )
                                        } else {
                                            None
                                        }
                                    }
                                    _ => None,
                                };

                                let ty = segment.ident.to_string();
                                variant.args.push((name, ty, generic_ty));
                            }
                            _ => panic!("#[args] requires an expression 'name: type'"),
                        }
                    }
                }
            }

            enum_variants.push(variant);
        }

        enum_variants
    } else {
        panic!("No enum variants found");
    };

    let mut account_structs = generate_accounts(&variants);
    account_structs.extend(generate_builders(&variants));

    account_structs
}

/// Generates a struct for each enum variant.
///
/// The struct will contain all shank annotated accounts and the impl block
/// will initialize them using the accounts iterators. It support the use of
/// optional accounts, which would generate an account field with an
/// `Option<AccountInfo<'a>>` type.
///
/// ```ignore
/// pub struct MyAccount<'a> {
///     my_first_account: solana_program::account_info::AccountInfo<'a>,
///     my_second_optional_account: Option<solana_program::account_info::AccountInfo<'a>>,
///     ..
/// }
/// impl<'a> MyAccount<'a> {
///     pub fn to_context(
///         accounts: &'a [solana_program::account_info::AccountInfo<'a>]
///     ) -> Result<Context<'a, Self>, solana_program::sysvar::slot_history::ProgramError> {
///         let account_info_iter = &mut accounts.iter();
///
///         let my_first_account = solana_program::account_info::next_account_info(account_info_iter)?;
///
///         ..
///
///     }
/// }
/// ```
fn generate_accounts(variants: &[Variant]) -> TokenStream {
    // build the trait implementation
    let variant_structs = variants.iter().map(|variant| {
        let name = syn::parse_str::<syn::Ident>(&variant.name).unwrap();
        // accounts names
        let fields = variant.accounts.iter().map(|account| {
            let account_name = syn::parse_str::<syn::Ident>(format!("{}_info", &account.name).as_str()).unwrap();
            quote! { #account_name }
        });
        // accounts fields
        let struct_fields = variant.accounts.iter().map(|account| {
            let account_name = syn::parse_str::<syn::Ident>(format!("{}_info", &account.name).as_str()).unwrap();
            if account.optional {
                quote! {
                    pub #account_name: Option<&'a solana_program::account_info::AccountInfo<'a>>
                }
            } else {
                quote! {
                    pub #account_name:&'a solana_program::account_info::AccountInfo<'a>
                }
            }
        });
        // accounts initialization for the impl block
        let impl_fields = variant.accounts.iter().map(|account| {
            let account_name = syn::parse_str::<syn::Ident>(format!("{}_info", &account.name).as_str()).unwrap();
            if account.optional {
                quote! {
                    let #account_name = crate::processor::next_optional_account_info(account_info_iter)?;
                }
            } else {
                quote! {
                    let #account_name = solana_program::account_info::next_account_info(account_info_iter)?;
                }
            }
        });

        quote! {
            pub struct #name<'a> {
                #(#struct_fields,)*
            }
            impl<'a> #name<'a> {
                pub fn to_context(accounts: &'a [solana_program::account_info::AccountInfo<'a>]) -> Result<Context<'a, Self>, solana_program::sysvar::slot_history::ProgramError> {
                    let account_info_iter = &mut accounts.iter();

                    #(#impl_fields)*

                    let accounts = Self {
                        #(#fields,)*
                    };

                    Ok(Context {
                        accounts,
                        remaining_accounts: Vec::<&'a AccountInfo<'a>>::from_iter(account_info_iter),
                    })
                }
            }
        }
    });

    TokenStream::from(quote! {
        #(#variant_structs)*
    })
}

fn generate_builders(variants: &[Variant]) -> TokenStream {
    let mut default_pubkeys = HashMap::new();
    default_pubkeys.insert(
        "system_program".to_string(),
        syn::parse_str::<syn::ExprPath>("solana_program::system_program::ID").unwrap(),
    );
    default_pubkeys.insert(
        "spl_token_program".to_string(),
        syn::parse_str::<syn::ExprPath>("spl_token::ID").unwrap(),
    );
    default_pubkeys.insert(
        "spl_ata_program".to_string(),
        syn::parse_str::<syn::ExprPath>("spl_associated_token_account::ID").unwrap(),
    );
    default_pubkeys.insert(
        "sysvar_instructions".to_string(),
        syn::parse_str::<syn::ExprPath>("solana_program::sysvar::instructions::ID").unwrap(),
    );
    default_pubkeys.insert(
        "authorization_rules_program".to_string(),
        syn::parse_str::<syn::ExprPath>("mpl_token_auth_rules::ID").unwrap(),
    );

    // build the trait implementation
    let variant_structs = variants.iter().map(|variant| {
        let name = syn::parse_str::<syn::Ident>(&variant.name).unwrap();

        // struct block for the builder: this will contain both accounts and
        // args for the builder

        // accounts
        let struct_accounts = variant.accounts.iter().map(|account| {
            let account_name = syn::parse_str::<syn::Ident>(&account.name).unwrap();
            if account.optional {
                quote! {
                    pub #account_name: Option<solana_program::pubkey::Pubkey>
                }
            } else {
                quote! {
                    pub #account_name: solana_program::pubkey::Pubkey
                }
            }
        });

        // args
        let struct_args = variant.args.iter().map(|(name, ty, generic_ty)| {
            let ident_ty = syn::parse_str::<syn::Ident>(ty).unwrap();
            let arg_ty = if let Some(genetic_ty) = generic_ty {
                let arg_generic_ty = syn::parse_str::<syn::Ident>(genetic_ty).unwrap();
                quote! { #ident_ty<#arg_generic_ty> }
            } else {
                quote! { #ident_ty }
            };
            let arg_name = syn::parse_str::<syn::Ident>(name).unwrap();
              
            quote! {
                pub #arg_name: #arg_ty
            }
        });

        // builder block: this will have all accounts and args as optional fields
        // that need to be set before the build method is called

        // accounts
        let builder_accounts = variant.accounts.iter().map(|account| {
            let account_name = syn::parse_str::<syn::Ident>(&account.name).unwrap();
            quote! {
                pub #account_name: Option<solana_program::pubkey::Pubkey>
            }
        });

        // accounts initialization
        let builder_initialize_accounts = variant.accounts.iter().map(|account| {
            let account_name = syn::parse_str::<syn::Ident>(&account.name).unwrap();
            quote! {
                #account_name: None
            }
        });

        // args
        let builder_args = variant.args.iter().map(|(name, ty, generic_ty)| {
            let ident_ty = syn::parse_str::<syn::Ident>(ty).unwrap();
            let arg_ty = if let Some(genetic_ty) = generic_ty {
                let arg_generic_ty = syn::parse_str::<syn::Ident>(genetic_ty).unwrap();
                quote! { #ident_ty<#arg_generic_ty> }
            } else {
                quote! { #ident_ty }
            };
            let arg_name = syn::parse_str::<syn::Ident>(name).unwrap();

            quote! {
                pub #arg_name: Option<#arg_ty>
            }
        });

        // args initialization
        let builder_initialize_args = variant.args.iter().map(|(name, _ty, _generi_ty)| {
            let arg_name = syn::parse_str::<syn::Ident>(name).unwrap();
            quote! {
                #arg_name: None
            }
        });

        // account setter methods
        let builder_accounts_methods = variant.accounts.iter().map(|account| {
            let account_name = syn::parse_str::<syn::Ident>(&account.name).unwrap();
            quote! {
                pub fn #account_name(&mut self, #account_name: solana_program::pubkey::Pubkey) -> &mut Self {
                    self.#account_name = Some(#account_name);
                    self
                }
            }
        });

        // args setter methods
        let builder_args_methods = variant.args.iter().map(|(name, ty, generic_ty)| {
            let ident_ty = syn::parse_str::<syn::Ident>(ty).unwrap();
            let arg_ty = if let Some(genetic_ty) = generic_ty {
                let arg_generic_ty = syn::parse_str::<syn::Ident>(genetic_ty).unwrap();
                quote! { #ident_ty<#arg_generic_ty> }
            } else {
                quote! { #ident_ty }
            };
            let arg_name = syn::parse_str::<syn::Ident>(name).unwrap();

            quote! {
                pub fn #arg_name(&mut self, #arg_name: #arg_ty) -> &mut Self {
                    self.#arg_name = Some(#arg_name);
                    self
                }
            }
        });

        // required accounts
        let required_accounts = variant.accounts.iter().map(|account| {
            let account_name = syn::parse_str::<syn::Ident>(&account.name).unwrap();

            if account.optional {
                quote! {
                    #account_name: self.#account_name
                }
            } else {
                // are we dealing with a default pubkey?
                if default_pubkeys.contains_key(&account.name) {
                    let pubkey = default_pubkeys.get(&account.name).unwrap();
                    // we add the default key as the fallback value
                    quote! {
                        #account_name: self.#account_name.unwrap_or(#pubkey)
                    }
                }
                else {
                    // if not a default pubkey, we will need to have it set
                    quote! {
                        #account_name: self.#account_name.ok_or(concat!(stringify!(#account_name), " is not set"))?
                    }
                }
            }
        });

        // required args
        let required_args = variant.args.iter().map(|(name, _ty, _generic_ty)| {
            let arg_name = syn::parse_str::<syn::Ident>(name).unwrap();
            quote! {
                #arg_name: self.#arg_name.clone().ok_or(concat!(stringify!(#arg_name), " is not set"))?
            }
        });

        // args parameter list
        let args = if let Some(args) = &variant.tuple {
            let arg_ty = syn::parse_str::<syn::Ident>(args).unwrap();
            quote! { &mut self, args: #arg_ty }
        } else {
            quote! { &mut self }
        };

        // instruction args
        let instruction_args = if let Some(args) = &variant.tuple {
            let arg_ty = syn::parse_str::<syn::Ident>(args).unwrap();
            quote! { pub args: #arg_ty, }
        } else {
            quote! { }
        };

        // required instruction args
        let required_instruction_args = if variant.tuple.is_some() {
            quote! { args, }
        } else {
            quote! { }
        };

        // builder name
        let builder_name = syn::parse_str::<syn::Ident>(&format!("{}Builder", name)).unwrap();

        quote! {
            pub struct #name {
                #(#struct_accounts,)*
                #(#struct_args,)*
                #instruction_args
            }

            pub struct #builder_name {
                #(#builder_accounts,)*
                #(#builder_args,)*
            }

            impl #builder_name {
                pub fn new() -> Box<#builder_name> {
                    Box::new(#builder_name {
                        #(#builder_initialize_accounts,)*
                        #(#builder_initialize_args,)*
                    })
                }

                #(#builder_accounts_methods)*
                #(#builder_args_methods)*

                pub fn build(#args) -> Result<Box<#name>, Box<dyn std::error::Error>> {
                    Ok(Box::new(#name {
                        #(#required_accounts,)*
                        #(#required_args,)*
                        #required_instruction_args
                    }))
                }
            }
        }
    });

    TokenStream::from(quote! {
        pub mod builders {
            use super::*;

            #(#variant_structs)*
        }
    })
}
