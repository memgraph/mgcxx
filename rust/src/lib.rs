use std::io::{Error, ErrorKind};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};

#[cxx::bridge]
mod ffi {
    #[namespace = "text_search"]
    struct TextInput {
        data: String,
    }
    #[namespace = "text_search"]
    struct Context {
        tantivyContext: Box<TantivyContext>,
    }
    #[namespace = "text_search"]
    struct SearchInput {
        query: String,
    }
    #[namespace = "text_search"]
    struct SearchOutput {
        doc_ids: Vec<u64>,
    }

    #[namespace = "cxxtantivy"]
    extern "Rust" {
        type TantivyContext;
        fn init() -> Result<Context>;
        fn add(context: &mut Context, input: &TextInput) -> bool;
        fn search(context: &mut Context, input: &SearchInput) -> Result<SearchOutput>;
    }
}

pub struct TantivyContext {
    pub schema: Schema,
    pub index: Index,
    pub index_writer: IndexWriter,
}

fn add(context: &mut ffi::Context, input: &ffi::TextInput) -> bool {
    let props = context.tantivyContext.schema.get_field("props").unwrap();
    match context
        .tantivyContext
        .index_writer
        .add_document(doc!(props => input.data.clone()))
    {
        Ok(_) => {
            // pass
        }
        Err(_) => return false,
    };
    match context.tantivyContext.index_writer.commit() {
        Ok(_) => return true,
        Err(_) => return false,
    }
}

fn search(
    context: &mut ffi::Context,
    input: &ffi::SearchInput,
) -> Result<ffi::SearchOutput, std::io::Error> {
    let reader = match context
        .tantivyContext
        .index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()
    {
        Ok(r) => r,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to read (reader init failed): {}", e),
            ));
        }
    };
    let searcher = reader.searcher();
    let props = context.tantivyContext.schema.get_field("props").unwrap();
    let query_parser = QueryParser::for_index(&context.tantivyContext.index, vec![props]);
    let query = match query_parser.parse_query(&input.query) {
        Ok(q) => q,
        Err(_e) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Unable to create search query",
            ));
        }
    };
    let top_docs = match searcher.search(&query, &TopDocs::with_limit(10)) {
        Ok(docs) => docs,
        Err(_e) => {
            return Err(Error::new(ErrorKind::Other, "Unable to perform search"));
        }
    };
    let mut doc_ids: Vec<u64> = vec![];
    for (_score, doc_address) in top_docs {
        doc_ids.push(doc_address.doc_id.into());
    }
    Ok(ffi::SearchOutput { doc_ids })
}

fn init() -> Result<ffi::Context, std::io::Error> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("props", TEXT | STORED);
    let schema = schema_builder.build();

    let index_path = std::path::Path::new("tantivy_index");
    if !index_path.exists() {
        match std::fs::create_dir(index_path) {
            Ok(_) => {
                println!("tantivy_index folder created");
            }
            Err(_) => {
                panic!("Failed to create tantivy_index folder");
            }
        }
    }

    let mmap_directory = MmapDirectory::open(&index_path).unwrap();
    let index = match Index::open_or_create(mmap_directory, schema.clone()) {
        Ok(index) => index,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to initialize index -> {}", e),
            ));
        }
    };
    // NOTE: The following assert is not needed because if the schema is wrong
    // Index::open_or_create is going to fail.
    // assert!(index.schema() == schema, "Schema loaded from tantivy_index does NOT match.");
    // TODO(gitbuda): Implement text search backward compatiblity because of possible schema changes.

    let index_writer: IndexWriter = match index.writer(50_000_000) {
        Ok(writer) => writer,
        Err(_e) => {
            return Err(Error::new(ErrorKind::Other, "Unable to initialize writer"));
        }
    };
    Ok(ffi::Context {
        tantivyContext: Box::new(TantivyContext {
            schema,
            index,
            index_writer,
        }),
    })
}
