use log::debug;
use serde_json::{to_string, Value};
use std::io::{Error, ErrorKind};
use tantivy::aggregation::agg_req::Aggregations;
use tantivy::aggregation::agg_result::AggregationResults;
use tantivy::aggregation::AggregationCollector;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexWriter, ReloadPolicy};

// NOTE: Result<T> == Result<T,std::io::Error>.
#[cxx::bridge(namespace = "cxxtantivy")]
mod ffi {
    struct Context {
        tantivyContext: Box<TantivyContext>,
    }

    // NOTE: The input struct is / should be aligned with the schema.
    // NOTE: DocumentInputX (and X suffix in general) is for test/bench purposes.
    // NOTE: Having a specific input object under ffi is a problem for general solution.
    // NOTE: The following are metadata fields required by Memgraph
    //   metadata: String,
    //   gid: u64,
    //   txid: u64,
    //   deleted: bool,
    //   is_node: bool,
    // props: String, // TODO(gitbuda): Consider using https://cxx.rs/binding/cxxstring.html (c++
    //                // string on Rust stack).
    struct DocumentInput1 {
        metadata_and_data: String,
    }
    struct DocumentInput2 {
        gid: u64,
        data: String,
    }
    struct DocumentInput3 {
        metadata_and_data: String, // TODO(gitbuda): Test CxxString
    }
    struct SearchInput {
        search_query: String,
        aggregation_query: String,
        // TODO(gitbuda): Add stuff like skip & limit.
    }

    struct DocumentOutput {
        data: String, // TODO(gitbuda): Here should be Option but it's not supported in cxx.
    }
    struct SearchOutput {
        docs: Vec<DocumentOutput>,
        // TODO(gitbuda): Add stuff like page (skip, limit).
    }

    // NOTE: Since return type is Result<T>, always return Result<Something>.
    extern "Rust" {
        type TantivyContext;
        fn drop_index(name: &String) -> Result<()>;
        fn init() -> Result<()>;
        fn create_index1(name: &String) -> Result<Context>;
        fn create_index2(name: &String) -> Result<Context>;
        fn aggregate(context: &mut Context, input: &SearchInput) -> Result<DocumentOutput>;
        fn search(context: &mut Context, input: &SearchInput) -> Result<SearchOutput>;
        fn find(context: &mut Context, input: &SearchInput) -> Result<SearchOutput>;
        fn add1(context: &mut Context, input: &DocumentInput1, skip_commit: bool) -> Result<()>;
        fn add2(context: &mut Context, input: &DocumentInput2, skip_commit: bool) -> Result<()>;
        fn commit(context: &mut Context) -> Result<()>;
        fn rollback(context: &mut Context) -> Result<()>;
    }
}

pub struct TantivyContext {
    // TODO(gitbuda): Consider prefetching schema fields into context (measure first).
    pub schema: Schema,
    pub index: Index,
    pub index_writer: IndexWriter,
}

fn rollback(context: &mut ffi::Context) -> Result<(), std::io::Error> {
    let index_writer = &mut context.tantivyContext.index_writer;
    match index_writer.rollback() {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to rollback -> {}", e),
            ));
        }
    }
}

fn commit_(index_writer: &mut IndexWriter) -> Result<(), std::io::Error> {
    match index_writer.commit() {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to commit -> {}", e),
            ));
        }
    }
}

fn commit(context: &mut ffi::Context) -> Result<(), std::io::Error> {
    let index_writer = &mut context.tantivyContext.index_writer;
    commit_(index_writer)
}

fn add_document(
    index_writer: &mut IndexWriter,
    document: Document,
    skip_commit: bool,
) -> Result<(), std::io::Error> {
    match index_writer.add_document(document) {
        Ok(_) => {
            if skip_commit {
                return Ok(());
            } else {
                commit_(index_writer)
            }
        }
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to add document -> {}", e),
            ));
        }
    }
}

fn add1(
    context: &mut ffi::Context,
    input: &ffi::DocumentInput1,
    skip_commit: bool,
) -> Result<(), std::io::Error> {
    let schema = &context.tantivyContext.schema;
    let index_writer = &mut context.tantivyContext.index_writer;
    // TODO(gitbuda): schema.parse_document > TantivyDocument::parse_json (LATEST UNSTABLE)
    let document = match schema.parse_document(&input.metadata_and_data) {
        Ok(json) => json,
        Err(e) => panic!("failed to parser metadata {}", e),
    };
    add_document(index_writer, document, skip_commit)
}

fn add2(
    context: &mut ffi::Context,
    input: &ffi::DocumentInput2,
    skip_commit: bool,
) -> Result<(), std::io::Error> {
    let schema = &context.tantivyContext.schema;
    let index_writer = &mut context.tantivyContext.index_writer;
    let gid_field = schema.get_field("gid").unwrap();
    let data_field = schema.get_field("data").unwrap();
    let document = doc!(gid_field => input.gid, data_field => input.data.clone());
    add_document(index_writer, document, skip_commit)
}

fn aggregate(
    context: &mut ffi::Context,
    input: &ffi::SearchInput,
) -> Result<ffi::DocumentOutput, std::io::Error> {
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

    let data_field = schema.get_field("data").unwrap();
    let query_parser = QueryParser::for_index(index, vec![data_field]);
    let query = match query_parser.parse_query(&input.search_query) {
        Ok(q) => q,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to create search query {}", e),
            ));
        }
    };

    let searcher = reader.searcher();
    let agg_req: Aggregations = serde_json::from_str(&input.aggregation_query)?;
    let collector = AggregationCollector::from_aggs(agg_req, Default::default());
    let agg_res: AggregationResults = searcher.search(&query, &collector).unwrap();
    let res: Value = serde_json::to_value(agg_res)?;
    Ok(ffi::DocumentOutput {
        data: res.to_string(),
    })
}

fn find(
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
    let data_field = schema.get_field("data").unwrap();
    let query_parser = QueryParser::for_index(index, vec![gid_field]);
    let query = match query_parser.parse_query(&input.search_query) {
        Ok(q) => q,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to create search query {}", e),
            ));
        }
    };
    let top_docs = match reader.searcher().search(&query, &TopDocs::with_limit(10)) {
        Ok(docs) => docs,
        Err(_e) => {
            return Err(Error::new(ErrorKind::Other, "Unable to perform search"));
        }
    };
    let mut docs: Vec<ffi::DocumentOutput> = vec![];
    for (_score, doc_address) in top_docs {
        let doc = match reader.searcher().doc(doc_address) {
            Ok(d) => d,
            Err(_) => {
                panic!("Unable to find document returned by the search query.");
            }
        };
        // let data = doc.get_first(data_field).unwrap().as_json().unwrap();
        let data = doc.get_first(data_field).unwrap().as_text().unwrap();
        // let data = schema.to_json(&doc);
        docs.push(ffi::DocumentOutput {
            data: match to_string(&data) {
                Ok(s) => s,
                Err(_e) => {
                    panic!("stored data not JSON");
                }
            },
        });
    }
    Ok(ffi::SearchOutput { docs })
}

// TODO(gitbuda): Test/figure_out how to get all properties from the schema.
// let fields = schema.fields();
// TODO(gitbuda): Test fuzzy searches
// let term = Term::from_field_text(data_field, &input.search_query);
// let query = FuzzyTermQuery::new(term, 2, true);
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
    let metadata_field = schema.get_field("metadata").unwrap();
    let data_field = schema.get_field("data").unwrap();
    let query_parser = QueryParser::for_index(index, vec![metadata_field]);
    let query = match query_parser.parse_query(&input.search_query) {
        Ok(q) => q,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to create search query {}", e),
            ));
        }
    };
    let top_docs = match reader.searcher().search(&query, &TopDocs::with_limit(10)) {
        Ok(docs) => docs,
        Err(_e) => {
            return Err(Error::new(ErrorKind::Other, "Unable to perform search"));
        }
    };
    let mut docs: Vec<ffi::DocumentOutput> = vec![];
    for (_score, doc_address) in top_docs {
        let doc = match reader.searcher().doc(doc_address) {
            Ok(d) => d,
            Err(_) => {
                panic!("Unable to find document returned by the search query.");
            }
        };
        // let metadata = doc.get_first(metadata_field).unwrap().as_json().unwrap();
        let data = doc.get_first(data_field).unwrap().as_json().unwrap();
        // let data = schema.to_json(&doc);
        docs.push(ffi::DocumentOutput {
            data: match to_string(&data) {
                Ok(s) => s,
                Err(_e) => {
                    panic!("stored data not JSON");
                }
            },
        });
    }
    Ok(ffi::SearchOutput { docs })
}

fn drop_index(name: &String) -> Result<(), std::io::Error> {
    let index_path = std::path::Path::new(name);
    if index_path.exists() {
        match std::fs::remove_dir_all(index_path) {
            Ok(_) => {
                debug!("tantivy_index removed");
            }
            Err(_) => {
                // panic!("Failed to remove tantivy_index folder {}", e);
            }
        }
    } else {
        debug!("tantivy_index folder doesn't exist");
    }
    Ok(())
}

fn init() -> Result<(), std::io::Error> {
    let log_init_res = env_logger::try_init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    if let Err(e) = log_init_res {
        println!("failed to initialize logger: {e:?}");
    }
    Ok(())
}

//// CREATE INDEX ////
// NOTE: TEXT is required to be able to search.
// TODO(gitbuda): Add mappings as a String to create_index.
// TODO(gitbuda): Expose index path to be configurable on the C++ side.
// TODO(gitbuda): Don't panic because if index can't be created -> just return to the user.
// TODO(gitbuda): Test what's the tradeoff between searching STRING vs JSON TEXT, how does the
// query look like?
// TODO(gitbuda): Benchmark SLOW vs FAST on data, consider this making the configurable by the
// user -> what's the tradeoff?

fn ensure_index_dir_structure(name: &String, schema: &Schema) -> Result<Index, std::io::Error> {
    let index_path = std::path::Path::new(name);
    if !index_path.exists() {
        match std::fs::create_dir(index_path) {
            Ok(_) => {
                debug!("{:?} folder created", index_path);
            }
            Err(_) => {
                panic!("Failed to create {:?} folder", index_path);
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
    Ok(index)
}
fn create_index_writter(index: &Index) -> Result<IndexWriter, std::io::Error> {
    let index_writer: IndexWriter = match index.writer(50_000_000) {
        Ok(writer) => writer,
        Err(_e) => {
            // TODO(gitbuda): This message won't be intuitive to the user -> rewrite.
            return Err(Error::new(ErrorKind::Other, "Unable to initialize writer"));
        }
    };
    Ok(index_writer)
}

fn create_index1(name: &String) -> Result<ffi::Context, std::io::Error> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_json_field("metadata", STORED | TEXT | FAST);
    schema_builder.add_json_field("data", STORED | TEXT | FAST);
    let schema = schema_builder.build();
    let index = ensure_index_dir_structure(name, &schema)?;
    let index_writer = create_index_writter(&index)?;
    Ok(ffi::Context {
        tantivyContext: Box::new(TantivyContext {
            schema,
            index,
            index_writer,
        }),
    })
}

fn create_index2(name: &String) -> Result<ffi::Context, std::io::Error> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_u64_field("gid", FAST | STORED | INDEXED);
    // TODO(gitbuda): Be careful, here is just a plain text use JSON instead.
    schema_builder.add_text_field("data", STORED | TEXT | FAST);
    let schema = schema_builder.build();
    let index = ensure_index_dir_structure(name, &schema)?;
    let index_writer = create_index_writter(&index)?;
    Ok(ffi::Context {
        tantivyContext: Box::new(TantivyContext {
            schema,
            index,
            index_writer,
        }),
    })
}
