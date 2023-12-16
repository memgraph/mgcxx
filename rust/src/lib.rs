use std::io::{Error, ErrorKind};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};

// NOTE: Result<T> == Result<T,std::io::Error>.
#[cxx::bridge]
mod ffi {
    #[namespace = "text_search"]
    struct Context {
        tantivyContext: Box<TantivyContext>,
    }

    #[namespace = "text_search"]
    struct Element {
        gid: u64,
        txid: u64,
        deleted: bool,
        is_node: bool,
        props: String,
    }

    #[namespace = "text_search"]
    struct DocumentInput {
        // TODO(gitbuda): What's the best type here? String or JSON
        data: Element,
    }

    #[namespace = "text_search"]
    struct SearchInput {
        query: String,
        // TODO(gitbuda): Add stuff like skip & limit.
    }

    #[namespace = "text_search"]
    struct SearchOutput {
        docs: Vec<Element>,
        // TODO(gitbuda): Add stuff like page (skip, limit).
    }

    // NOTE: Since return type is Result<T>, always return Result<Something>.
    #[namespace = "cxxtantivy"]
    extern "Rust" {
        type TantivyContext;
        fn drop_index() -> Result<()>;
        fn init() -> Result<Context>;
        fn add(context: &mut Context, input: &DocumentInput) -> Result<()>;
        fn search(context: &mut Context, input: &SearchInput) -> Result<SearchOutput>;
    }
}

pub struct TantivyContext {
    // TODO(gitbuda): Consider prefetching schema fields into context (measure first).
    pub schema: Schema,
    pub index: Index,
    pub index_writer: IndexWriter,
}

fn add(context: &mut ffi::Context, input: &ffi::DocumentInput) -> Result<(), std::io::Error> {
    let schema = &context.tantivyContext.schema;
    let index_writer = &mut context.tantivyContext.index_writer;

    let gid_field = schema.get_field("gid").unwrap();
    let txid_field = schema.get_field("txid").unwrap();
    let deleted_field = schema.get_field("deleted").unwrap();
    let is_node_field = schema.get_field("is_node").unwrap();
    let props_field = schema.get_field("props").unwrap();

    match index_writer.add_document(doc!(
            gid_field => input.data.gid,
            txid_field => input.data.txid,
            deleted_field => input.data.deleted,
            is_node_field => input.data.is_node,
            props_field => input.data.props.clone()))
    {
        Ok(_) => match index_writer.commit() {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Unable to commit adding document -> {}", e),
                ));
            }
        },
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to add document -> {}", e),
            ));
        }
    };
}

fn search(
    context: &mut ffi::Context,
    input: &ffi::SearchInput,
) -> Result<ffi::SearchOutput, std::io::Error> {
    let index = &context.tantivyContext.index;
    let schema = &context.tantivyContext.schema;

    let reader = match index
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

    let gid_field = schema.get_field("gid").unwrap();
    let txid_field = schema.get_field("txid").unwrap();
    let deleted_field = schema.get_field("deleted").unwrap();
    let is_node_field = schema.get_field("is_node").unwrap();
    let props_field = schema.get_field("props").unwrap();

    let query_parser = QueryParser::for_index(index, vec![props_field]);
    let query = match query_parser.parse_query(&input.query) {
        Ok(q) => q,
        Err(_e) => {
            return Err(Error::new(
                ErrorKind::Other,
                "Unable to create search query",
            ));
        }
    };

    let top_docs = match reader.searcher().search(&query, &TopDocs::with_limit(10)) {
        Ok(docs) => docs,
        Err(_e) => {
            return Err(Error::new(ErrorKind::Other, "Unable to perform search"));
        }
    };
    let mut docs: Vec<ffi::Element> = vec![];
    for (_score, doc_address) in top_docs {
        let doc = match reader.searcher().doc(doc_address) {
            Ok(d) => d,
            Err(_) => {
                panic!("Unable to find document returned by the search query.");
            }
        };
        let gid = doc.get_first(gid_field).unwrap().as_u64().unwrap();
        let txid = doc.get_first(txid_field).unwrap().as_u64().unwrap();
        let deleted = doc.get_first(deleted_field).unwrap().as_bool().unwrap();
        let is_node = doc.get_first(is_node_field).unwrap().as_bool().unwrap();
        let props = doc.get_first(props_field).unwrap().as_text().unwrap();
        docs.push(ffi::Element {
            gid,
            txid,
            deleted,
            is_node,
            props: props.to_string(),
        });
    }
    Ok(ffi::SearchOutput { docs })
}

fn drop_index() -> Result<(), std::io::Error> {
    let index_path = std::path::Path::new("tantivy_index");
    if index_path.exists() {
        match std::fs::remove_dir_all(index_path) {
            Ok(_) => {
                println!("tantivy index removed");
            }
            Err(_) => {
                panic!("Failed to remove tantivy_index folder");
            }
        }
    } else {
        return Err(Error::new(
            ErrorKind::Other,
            format!("Index doesn't not exist."),
        ));
    }
    Ok(())
}

fn init() -> Result<ffi::Context, std::io::Error> {
    // TODO(gitbuda): Expose elements to configure schema on the C++ side.
    let mut schema_builder = Schema::builder();
    schema_builder.add_u64_field("gid", FAST | STORED);
    schema_builder.add_u64_field("txid", FAST | STORED);
    schema_builder.add_bool_field("deleted", FAST | STORED);
    schema_builder.add_bool_field("is_node", FAST | STORED);
    schema_builder.add_text_field("props", TEXT | STORED);
    let schema = schema_builder.build();

    // TODO(gitbuda): Expose index path to be configurable on the C++ side.
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
