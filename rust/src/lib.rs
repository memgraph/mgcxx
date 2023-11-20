use std::io::{Error, ErrorKind};
use tantivy::collector::TopDocs;
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
    // TODO(gitbuda): Manage text search index folder creation (create folder).
    let index_path = std::path::Path::new("tantivy");
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("props", TEXT | STORED);
    let schema = schema_builder.build();
    // TODO(gitbuda): Skip index creation in case folder is alredy there.
    let index = match Index::create_in_dir(&index_path, schema.clone()) {
        Ok(index) => index,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to initialize index: {}", e),
            ));
        }
    };
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
