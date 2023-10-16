use std::{fs::File, io::Write};

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span};
use quote::quote;

pub fn create_store(name: String) {
    let name = pluralizer::pluralize(&name, 2, false).to_case(Case::Lower);
    let singular = pluralizer::pluralize(&name, 1, false).to_case(Case::Lower);

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
        use crate::repositories::accounts::models::Account;
        use async_trait::async_trait;
        use flair_core::store::snare::DBConnection;
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
        use flair_derive::Ensnare;
        use sqlx::FromRow;

        #[derive(Clone, Debug, FromRow, Ensnare)]
        pub struct #model_ident {
            pub id: i64,
        }
    }
    .to_string();

    let store_implementation = quote! {
        use async_trait::async_trait;
        use flair_core::store::snare::{DBConnection, Ensnared};

        use super::contract::#repository_contract;
        use super::models::#model_ident;

        pub struct #repository {}

        #[async_trait]
        impl #repository_contract<sqlx::MySql> for #repository {
            async fn #get_function<'c, C: DBConnection<'c>, E: From<sqlx::Error>>(
                conn: C,
                id: i64,
            ) -> Result<Option<#model_ident>, E> {
                let #name = sqlx::query_as::<_, #model_ident>("SELECT * FROM name WHERE id = ?")
                    .bind(id)
                    .fetch_optional(conn)
                    .await?;
                Ok(#name)
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
            store_mod,
        ),
        (
            format!(
                "store/src/repositories/{}/model.rs",
                name.to_case(Case::Snake)
            ),
            store_model,
        ),
        (
            format!(
                "store/src/repositories/{}/contract.rs",
                name.to_case(Case::Snake)
            ),
            store_contract,
        ),
        (
            format!(
                "store/src/repositories/{}/implementation.rs",
                name.to_case(Case::Snake)
            ),
            store_implementation,
        ),
    ];

    for (file_name, file_contents) in file_with_contents.iter() {
        let mut file = File::create(file_name).expect("Could not create controller file");
        file.write_all(file_contents.as_bytes())
            .expect("Could not write to controller file");
    }
}
