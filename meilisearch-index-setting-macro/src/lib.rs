use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro2::TokenTree;
use quote::quote;
use syn::parse_macro_input;
use syn::spanned::Spanned;

#[proc_macro_derive(IndexConfig, attributes(index_config))]
pub fn generate_index_settings(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);

    let fields: &syn::Fields = match ast.data {
        syn::Data::Struct(ref data) => &data.fields,
        _ => {
            return proc_macro::TokenStream::from(
                syn::Error::new(ast.ident.span(), "Applicable only to struct").to_compile_error(),
            );
        }
    };

    let struct_ident = &ast.ident;

    let index_config_implementation = get_index_config_implementation(struct_ident, fields);
    proc_macro::TokenStream::from(quote! {
        #index_config_implementation
    })
}

fn get_index_config_implementation(
    struct_ident: &syn::Ident,
    fields: &syn::Fields,
) -> proc_macro2::TokenStream {
    let mut attribute_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut primary_key_attribute: String = "".to_string();
    let mut distinct_key_attribute: String = "".to_string();
    let mut displayed_attributes: Vec<String> = vec![];
    let mut searchable_attributes: Vec<String> = vec![];
    let mut filterable_attributes: Vec<String> = vec![];
    let mut sortable_attributes: Vec<String> = vec![];
    let valid_attribute_names = std::collections::HashSet::from([
        "displayed",
        "searchable",
        "filterable",
        "sortable",
        "primary_key",
        "distinct",
    ]);

    let index_name = struct_ident
        .to_string()
        .from_case(Case::UpperCamel)
        .to_case(Case::Snake);

    for field in fields {
        let attribute_list_result =
            extract_all_attr_values(&field.attrs, &mut attribute_set, &valid_attribute_names);

        match attribute_list_result {
            Ok(attribute_list) => {
                for attribute in attribute_list {
                    match attribute.as_str() {
                        "displayed" => {
                            displayed_attributes.push(field.ident.clone().unwrap().to_string())
                        }
                        "searchable" => {
                            searchable_attributes.push(field.ident.clone().unwrap().to_string())
                        }
                        "filterable" => {
                            filterable_attributes.push(field.ident.clone().unwrap().to_string())
                        }
                        "sortable" => {
                            sortable_attributes.push(field.ident.clone().unwrap().to_string())
                        }
                        "primary_key" => {
                            primary_key_attribute = field.ident.clone().unwrap().to_string()
                        }
                        "distinct" => {
                            distinct_key_attribute = field.ident.clone().unwrap().to_string()
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                return e;
            }
        }
    }

    let primary_key_token: proc_macro2::TokenStream = if primary_key_attribute.is_empty() {
        quote! {
            ::std::option::Option::None
        }
    } else {
        quote! {
            ::std::option::Option::Some(#primary_key_attribute)
        }
    };

    let display_attr_tokens =
        get_settings_token_for_list(&displayed_attributes, "with_displayed_attributes");
    let sortable_attr_tokens =
        get_settings_token_for_list(&sortable_attributes, "with_sortable_attributes");
    let filterable_attr_tokens =
        get_settings_token_for_list(&filterable_attributes, "with_filterable_attributes");
    let searchable_attr_tokens =
        get_settings_token_for_list(&searchable_attributes, "with_searchable_attributes");
    let distinct_attr_token =
        get_settings_token_for_string(&distinct_key_attribute, "with_distinct_attribute");

    quote! {
        #[::meilisearch_sdk::macro_helper::async_trait]
        impl ::meilisearch_sdk::documents::IndexConfig for #struct_ident {
            const INDEX_STR: &'static str = #index_name;

            fn generate_settings() -> ::meilisearch_sdk::settings::Settings {
            ::meilisearch_sdk::settings::Settings::new()
            #display_attr_tokens
            #sortable_attr_tokens
            #filterable_attr_tokens
            #searchable_attr_tokens
            #distinct_attr_token
        }

         async fn generate_index(client: &::meilisearch_sdk::client::Client) -> std::result::Result<::meilisearch_sdk::indexes::Index, ::meilisearch_sdk::tasks::Task> {
            return client.create_index(#index_name, #primary_key_token)
                .await.unwrap()
                .wait_for_completion(&client, ::std::option::Option::None, ::std::option::Option::None)
                .await.unwrap()
                .try_make_index(&client);
            }
        }
    }
}

fn extract_all_attr_values(
    attrs: &[syn::Attribute],
    attribute_set: &mut std::collections::HashSet<String>,
    valid_attribute_names: &std::collections::HashSet<&str>,
) -> std::result::Result<Vec<String>, proc_macro2::TokenStream> {
    let mut attribute_names: Vec<String> = vec![];
    let mut local_attribute_set: std::collections::HashSet<String> = HashSet::new();
    for attr in attrs {
        match attr.parse_meta() {
            std::result::Result::Ok(syn::Meta::List(list)) => {
                if !list.path.is_ident("index_config") {
                    continue;
                }
                for token_stream in attr.tokens.clone().into_iter() {
                    if let TokenTree::Group(group) = token_stream {
                        for token in group.stream() {
                            match token {
                                TokenTree::Punct(punct) => validate_punct(&punct)?,
                                TokenTree::Ident(ident) => {
                                    if ident == "primary_key"
                                        && attribute_set.contains("primary_key")
                                    {
                                        return std::result::Result::Err(
                                            syn::Error::new(
                                                ident.span(),
                                                "`primary_key` already exists",
                                            )
                                            .to_compile_error(),
                                        );
                                    }
                                    if ident == "distinct" && attribute_set.contains("distinct") {
                                        return std::result::Result::Err(
                                            syn::Error::new(
                                                ident.span(),
                                                "`distinct` already exists",
                                            )
                                            .to_compile_error(),
                                        );
                                    }

                                    if local_attribute_set.contains(ident.to_string().as_str()) {
                                        return std::result::Result::Err(
                                            syn::Error::new(
                                                ident.span(),
                                                format!("`{ident}` already exists for this field"),
                                            )
                                            .to_compile_error(),
                                        );
                                    }

                                    if !valid_attribute_names.contains(ident.to_string().as_str()) {
                                        return std::result::Result::Err(
                                                syn::Error::new(
                                                    ident.span(),
                                                    format!(
                                                        "Property `{ident}` does not exist for type `index_config`"
                                                    ),
                                                )
                                                    .to_compile_error(),
                                            );
                                    }
                                    attribute_names.push(ident.to_string());
                                    attribute_set.insert(ident.to_string());
                                    local_attribute_set.insert(ident.to_string());
                                }
                                _ => {
                                    return std::result::Result::Err(
                                        syn::Error::new(attr.span(), "Invalid parsing".to_string())
                                            .to_compile_error(),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            std::result::Result::Err(e) => {
                for token_stream in attr.tokens.clone().into_iter() {
                    if let TokenTree::Group(group) = token_stream {
                        for token in group.stream() {
                            if let TokenTree::Punct(punct) = token {
                                validate_punct(&punct)?
                            }
                        }
                    }
                }
                return std::result::Result::Err(
                    syn::Error::new(attr.span(), e.to_string()).to_compile_error(),
                );
            }
            _ => {}
        }
    }
    std::result::Result::Ok(attribute_names)
}

fn validate_punct(punct: &proc_macro2::Punct) -> std::result::Result<(), proc_macro2::TokenStream> {
    if punct.as_char() == ',' && punct.spacing() == proc_macro2::Spacing::Alone {
        return std::result::Result::Ok(());
    }
    std::result::Result::Err(
        syn::Error::new(punct.span(), "`,` expected".to_string()).to_compile_error(),
    )
}

fn get_settings_token_for_list(
    field_name_list: &Vec<String>,
    method_name: &str,
) -> proc_macro2::TokenStream {
    let string_attributes = field_name_list.iter().map(|attr| {
        quote! {
            #attr
        }
    });
    let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());

    if !field_name_list.is_empty() {
        quote! {
            .#method_ident([#(#string_attributes),*])
        }
    } else {
        quote! {
            .#method_ident(::std::iter::empty::<&str>())
        }
    }
}

fn get_settings_token_for_string(
    field_name: &String,
    method_name: &str,
) -> proc_macro2::TokenStream {
    let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());

    if !field_name.is_empty() {
        quote! {
            .#method_ident(#field_name)
        }
    } else {
        proc_macro2::TokenStream::new()
    }
}
