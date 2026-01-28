/// Macro to define a language implementation with reduced boilerplate.
///
/// This macro generates:
/// - A struct with `language`, `definitions_query`, and `usages_query` fields
/// - A `new()` constructor that loads the tree-sitter grammar and queries
/// - A `LanguageSupport` trait implementation
///
/// # Usage
///
/// ```ignore
/// define_language! {
///     name: RustLanguage,
///     id: Rust,
///     display_name: "Rust",
///     extensions: ["rs"],
///     tree_sitter_language: tree_sitter_rust::LANGUAGE,
///     query_dir: "rust",
///     mappings: RUST_DEFINITION_MAPPINGS,
/// }
/// ```
///
/// For languages that use separate documentation (e.g., SQL COMMENT ON):
/// ```ignore
/// define_language! {
///     name: SqlLanguage,
///     id: Sql,
///     display_name: "Sql",
///     extensions: ["sql"],
///     tree_sitter_language: tree_sitter_sequel::LANGUAGE,
///     query_dir: "sql",
///     mappings: SQL_DEFINITION_MAPPINGS,
///     uses_separate_docs: true,
/// }
/// ```
///
/// For languages with a shared mappings constant (e.g., TypeScript and TSX):
/// ```ignore
/// define_language! {
///     name: TypeScriptReactLanguage,
///     id: TypeScriptReact,
///     display_name: "TypeScriptReact",
///     extensions: ["tsx"],
///     tree_sitter_language: tree_sitter_typescript::LANGUAGE_TSX,
///     query_dir: "typescript",
///     mappings: super::typescript::TYPESCRIPT_DEFINITION_MAPPINGS,
/// }
/// ```
#[macro_export]
macro_rules! define_language {
    // Base case without uses_separate_docs
    (
        name: $name:ident,
        id: $id:ident,
        display_name: $display:literal,
        extensions: [$($ext:literal),+ $(,)?],
        tree_sitter_language: $ts_lang:expr,
        query_dir: $query_dir:literal,
        mappings: $mappings:expr $(,)?
    ) => {
        $crate::define_language! {
            name: $name,
            id: $id,
            display_name: $display,
            extensions: [$($ext),+],
            tree_sitter_language: $ts_lang,
            query_dir: $query_dir,
            mappings: $mappings,
            uses_separate_docs: false,
        }
    };

    // Full case with uses_separate_docs
    (
        name: $name:ident,
        id: $id:ident,
        display_name: $display:literal,
        extensions: [$($ext:literal),+ $(,)?],
        tree_sitter_language: $ts_lang:expr,
        query_dir: $query_dir:literal,
        mappings: $mappings:expr,
        uses_separate_docs: $sep_docs:literal $(,)?
    ) => {
        pub struct $name {
            language: ::tree_sitter::Language,
            definitions_query: ::tree_sitter::Query,
            usages_query: ::tree_sitter::Query,
        }

        impl $name {
            pub fn new() -> ::anyhow::Result<Self> {
                use ::anyhow::Context;

                let language: ::tree_sitter::Language = $ts_lang.into();

                let definitions_query_src =
                    include_str!(concat!("../../queries/", $query_dir, "/definitions.scm"));
                let usages_query_src =
                    include_str!(concat!("../../queries/", $query_dir, "/usages.scm"));

                let definitions_query = ::tree_sitter::Query::new(&language, definitions_query_src)
                    .context(concat!("Failed to parse ", $display, " definitions query"))?;
                let usages_query = ::tree_sitter::Query::new(&language, usages_query_src)
                    .context(concat!("Failed to parse ", $display, " usages query"))?;

                Ok(Self {
                    language,
                    definitions_query,
                    usages_query,
                })
            }
        }

        impl $crate::language::traits::LanguageSupport for $name {
            fn id(&self) -> $crate::language::traits::LanguageId {
                $crate::language::traits::LanguageId::$id
            }

            fn name(&self) -> &'static str {
                $display
            }

            fn file_extensions(&self) -> &[&'static str] {
                &[$($ext),+]
            }

            fn tree_sitter_language(&self) -> &::tree_sitter::Language {
                &self.language
            }

            fn definitions_query(&self) -> &::tree_sitter::Query {
                &self.definitions_query
            }

            fn usages_query(&self) -> &::tree_sitter::Query {
                &self.usages_query
            }

            fn definition_mappings(&self) -> &[$crate::language::traits::SymbolKindMapping] {
                $mappings
            }

            fn uses_separate_docs(&self) -> bool {
                $sep_docs
            }
        }
    };
}
