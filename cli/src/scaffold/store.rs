use std::{
    fs::{self, File},
    io::Write,
};

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span};
use quote::quote;
use rust_format::{Formatter, RustFmt};

pub fn create_store(name: String) {
    let name = pluralizer::pluralize(&name, 2, false).to_case(Case::Lower);
    let singular = pluralizer::pluralize(&name, 1, false).to_case(Case::Lower);
    let formatter = RustFmt::default();

    let namespace = Ident::new(
        &format!("{}", name.to_case(Case::Snake))[..],
        Span::call_site(),
    );
    let repository = Ident::new(
        &format!("{}Repository", name.to_case(Case::Pascal))[..],
        Span::call_site(),
    );
    let repository_contract = Ident::new(
        &format!("{}RepositoryContract", name.to_case(Case::Pascal))[..],
        Span::call_site(),
    );
    let get_function = Ident::new(&format!("get_{}", singular)[..], Span::call_site());
    let model_ident = Ident::new(
        &format!("{}", singular.to_case(Case::Pascal))[..],
        Span::call_site(),
    );

    let store_mod = quote! {
        pub mod contract;
        pub mod implementation;
        pub mod models;
    }
    .to_string();

    let store_contract = quote! {
        use crate::repositories::#namespace::models::#model_ident;
        use async_trait::async_trait;
        use cali_core::store::snare::DBConnection;
        use sqlx::Database;

        #[async_trait]
        pub trait #repository_contract<DB: Database> {
            async fn #get_function<'c, C: DBConnection<'c>, E: From<sqlx::Error>>(
                conn: C,
                id: i64,
            ) -> Result<Option<#model_ident>, E>;

        }
    }
    .to_string();

    let store_model = quote! {
        use cali_derive::Ensnare;
        use sqlx::FromRow;

        #[derive(Clone, Debug, FromRow, Ensnare)]
        pub struct #model_ident {
            pub id: i64,
        }
    }
    .to_string();

    let select = format!("SELECT * FROM {} WHERE id = ?", namespace);

    let store_implementation = quote! {
        use async_trait::async_trait;
        use cali_core::store::snare::{DBConnection, Ensnared};

        use super::contract::#repository_contract;
        use super::models::#model_ident;

        pub struct #repository {}

        #[async_trait]
        impl #repository_contract<sqlx::MySql> for #repository {
            async fn #get_function<'c, C: DBConnection<'c>, E: From<sqlx::Error>>(
                conn: C,
                id: i64,
            ) -> Result<Option<#model_ident>, E> {
                let #namespace = sqlx::query_as::<_, #model_ident>(#select)
                    .bind(id)
                    .fetch_optional(conn)
                    .await?;
                Ok(#namespace)
            }
        }
    }
    .to_string();

    let file_with_contents = [
        (
            format!(
                "store/src/repositories/{}/mod.rs",
                name.to_case(Case::Snake)
            ),
            formatter.format_str(store_mod).unwrap(),
        ),
        (
            format!(
                "store/src/repositories/{}/models.rs",
                name.to_case(Case::Snake)
            ),
            formatter.format_str(store_model).unwrap(),
        ),
        (
            format!(
                "store/src/repositories/{}/contract.rs",
                name.to_case(Case::Snake)
            ),
            formatter.format_str(store_contract).unwrap(),
        ),
        (
            format!(
                "store/src/repositories/{}/implementation.rs",
                name.to_case(Case::Snake)
            ),
            formatter.format_str(store_implementation).unwrap(),
        ),
    ];

    fs::create_dir(format!("store/src/repositories/{}", name))
        .expect("Should be able to create new repository");

    for (file_name, file_contents) in file_with_contents.iter() {
        let mut file = File::create(file_name).expect("Could not create repository files");
        file.write_all(file_contents.as_bytes())
            .expect("Could not write to repository file");
    }
}
