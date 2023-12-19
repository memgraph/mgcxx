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

    // TODO(gitbuda): Having input Element object under ffi is a problem for general solution.
    // NOTE: This struct is / should be aligned with the schema.
    struct Element {
        data: String,
        // the following are metadata fields required by Memgraph
        // TODO(gitbuda): Maybe all the following can be JSON and FAST
        // metadata: String,
        // gid: u64,
        // txid: u64,
        // deleted: bool,
        // is_node: bool,
        // props: String, // TODO(gitbuda): Consider using https://cxx.rs/binding/cxxstring.html (c++
        //                // string on Rust stack).
        //                // TODO(gitbuda): Consider renanaming to data, because could be used in 2 cases:
        //                //     * PropertyStore - serialize -> data
        //                //     * SingleProperty - serialize -> data
        //                // OPTION A:
        //                //   * meta: CxxString
        //                //   * data: CxxString
    }

    struct DocumentInput1 {
        // TODO(gitbuda): What's the best type here? String or JSON
        data: String,
    }

    struct SearchInput {
        search_query: String,
        aggregation_query: String,
        // TODO(gitbuda): Add stuff like skip & limit.
    }

    struct SearchOutput {
        docs: Vec<Element>,
        // TODO(gitbuda): Add stuff like page (skip, limit).
    }

    struct AggregateOutput {
        data: String, // TODO(gitbuda): Here should be Option but it's not supported in cxx.
    }

    // NOTE: Since return type is Result<T>, always return Result<Something>.
    extern "Rust" {
        type TantivyContext;
        fn drop_index(name: &String) -> Result<()>;
        fn init() -> Result<()>;
        fn create_index(name: &String) -> Result<Context>;
        fn add1(context: &mut Context, input: &DocumentInput1) -> Result<()>;
        fn search(context: &mut Context, input: &SearchInput) -> Result<SearchOutput>;
        fn aggregate(context: &mut Context, input: &SearchInput) -> Result<AggregateOutput>;
    }
}

pub struct TantivyContext {
    // TODO(gitbuda): Consider prefetching schema fields into context (measure first).
    pub schema: Schema,
    pub index: Index,
    pub index_writer: IndexWriter,
}

fn add1(context: &mut ffi::Context, input: &ffi::DocumentInput1) -> Result<(), std::io::Error> {
    let schema = &context.tantivyContext.schema;
    let index_writer = &mut context.tantivyContext.index_writer;

    // let metadata_field = schema.get_field("metadata"). unwrap();
    // TODO(gitbuda): schema.parse_document > TantivyDocument::parse_json (LATEST)
    let document = match schema.parse_document(&input.data) {
        Ok(json) => json,
        Err(e) => panic!("failed to parser metadata {}", e),
    };
    // let gid_field = schema.get_field("gid").unwrap();
    // let txid_field = schema.get_field("txid").unwrap();
    // let deleted_field = schema.get_field("deleted").unwrap();
    // let is_node_field = schema.get_field("is_node").unwrap();
    // let props_field = schema.get_field("props").unwrap();

    match index_writer.add_document(document) {
        // match index_writer.add_document(doc!(
        //         metadata_field => metadata,
        //         // gid_field => input.data.gid,
        //         // txid_field => input.data.txid,
        //         // deleted_field => input.data.deleted,
        //         // is_node_field => input.data.is_node,
        //         props_field => input.data.props.clone()))
        // {
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

fn aggregate(
    context: &mut ffi::Context,
    input: &ffi::SearchInput,
) -> Result<ffi::AggregateOutput, std::io::Error> {
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

    let props_field = schema.get_field("props").unwrap();
    let query_parser = QueryParser::for_index(index, vec![props_field]);
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
    Ok(ffi::AggregateOutput {
        data: res.to_string(),
    })
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

    let metadata_field = schema.get_field("metadata").unwrap();
    // let gid_field = schema.get_field("gid").unwrap();
    // let txid_field = schema.get_field("txid").unwrap();
    // let deleted_field = schema.get_field("deleted").unwrap();
    // let is_node_field = schema.get_field("is_node").unwrap();
    let props_field = schema.get_field("props").unwrap();

    let query_parser = QueryParser::for_index(index, vec![props_field]);
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
    let mut docs: Vec<ffi::Element> = vec![];
    for (_score, doc_address) in top_docs {
        let doc = match reader.searcher().doc(doc_address) {
            Ok(d) => d,
            Err(_) => {
                panic!("Unable to find document returned by the search query.");
            }
        };
        // let gid = doc.get_first(gid_field).unwrap().as_u64().unwrap();
        // let txid = doc.get_first(txid_field).unwrap().as_u64().unwrap();
        // let deleted = doc.get_first(deleted_field).unwrap().as_bool().unwrap();
        // let is_node = doc.get_first(is_node_field).unwrap().as_bool().unwrap();
        let metadata = doc.get_first(metadata_field).unwrap().as_json().unwrap();
        let props = doc.get_first(props_field).unwrap().as_json().unwrap();
        let data = schema.to_json(&doc);
        docs.push(ffi::Element {
            data: match to_string(&data) {
                Ok(s) => s,
                Err(_e) => {
                    panic!("stored data not JSON");
                }
            },
            // gid,
            // txid,
            // deleted,
            // is_node,
            // props: props.to_string(),
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

fn create_index(name: &String) -> Result<ffi::Context, std::io::Error> {
    // TODO(gitbuda): Expose elements to configure schema on the C++ side.
    let mut schema_builder = Schema::builder();
    schema_builder.add_json_field("metadata", FAST | STORED);
    // schema_builder.add_u64_field("gid", FAST | STORED);
    // schema_builder.add_u64_field("txid", FAST | STORED);
    // schema_builder.add_bool_field("deleted", FAST | STORED);
    // schema_builder.add_bool_field("is_node", FAST | STORED);

    // NOTE: TEXT is required to be able to search here
    // TODO(gitbuda): Test what's the tradeoff between searching STRING vs JSON TEXT, how does the
    // query look like?
    // TODO(gitbuda): Benchmark SLOW vs FAST on props, consider this making the configurable by the
    // user -> what's the tradeoff?
    schema_builder.add_json_field("props", STORED | TEXT | FAST);
    let schema = schema_builder.build();

    // TODO(gitbuda): Expose index path to be configurable on the C++ side.
    let index_path = std::path::Path::new(name);
    if !index_path.exists() {
        match std::fs::create_dir(index_path) {
            Ok(_) => {
                debug!("tantivy_index folder created");
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
