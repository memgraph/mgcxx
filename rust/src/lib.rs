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
use tantivy::{Index, IndexWriter, ReloadPolicy};

// NOTE: Result<T> == Result<T,std::io::Error>.
#[cxx::bridge(namespace = "cxxtantivy")]
mod ffi {
    struct Context {
        tantivyContext: Box<TantivyContext>,
    }

    struct IndexConfig {
        mappings: String,
    }

    struct DocumentInput {
        /// JSON encoded string with data.
        /// Mappings inside IndexConfig defines how data will be handeled.
        data: String,
    }
    // NOTE: The input struct is / should be aligned with the schema.
    // NOTE: Having a specific input object under ffi is a challange for general solution.
    // NOTE: The following are metadata fields required by Memgraph
    //   metadata: String,
    //   gid: u64,
    //   txid: u64,
    //   deleted: bool,
    //   is_node: bool,
    // props: String, // TODO(gitbuda): Consider using https://cxx.rs/binding/cxxstring.html

    struct SearchInput {
        search_query: String,
        aggregation_query: String,
        // TODO(gitbuda): Add stuff like skip & limit.
    }

    struct DocumentOutput {
        data: String, // NOTE: Here should probably be Option but it's not supported in cxx.
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
        fn create_index(name: &String, config: &IndexConfig) -> Result<Context>;
        fn aggregate(context: &mut Context, input: &SearchInput) -> Result<DocumentOutput>;
        fn search(context: &mut Context, input: &SearchInput) -> Result<SearchOutput>;
        fn find(context: &mut Context, input: &SearchInput) -> Result<SearchOutput>;
        fn add(context: &mut Context, input: &DocumentInput, skip_commit: bool) -> Result<()>;
        fn commit(context: &mut Context) -> Result<()>;
        fn rollback(context: &mut Context) -> Result<()>;
    }
}

pub struct TantivyContext {
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

fn add(
    context: &mut ffi::Context,
    input: &ffi::DocumentInput,
    skip_commit: bool,
) -> Result<(), std::io::Error> {
    let schema = &context.tantivyContext.schema;
    let index_writer = &mut context.tantivyContext.index_writer;
    // TODO(gitbuda): schema.parse_document > TantivyDocument::parse_json (LATEST UNSTABLE)
    let document = match schema.parse_document(&input.data) {
        Ok(json) => json,
        Err(e) => panic!("failed to parser metadata {}", e),
    };
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
        let data = doc.get_first(data_field).unwrap().as_json().unwrap();
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
    // NOTE: If schema doesn't match, open_or_create is going to return an error.
    let index = match Index::open_or_create(mmap_directory, schema.clone()) {
        Ok(index) => index,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to initialize index -> {}", e),
            ));
        }
    };
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

// TODO(gitbuda): Implement full range of extract_schema options.
fn extract_schema(mappings: &serde_json::Map<String, Value>) -> Result<Schema, std::io::Error> {
    let mut schema_builder = Schema::builder();
    if let Some(properties) = mappings.get("properties") {
        if let Some(properties_map) = properties.as_object() {
            for (field_name, value) in properties_map {
                let field_type = match value.get("type") {
                    Some(r) => match r.as_str() {
                        Some(s) => s,
                        None => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                "field type should be a string",
                            ));
                        }
                    },
                    None => {
                        return Err(Error::new(ErrorKind::Other, "field should have a type"));
                    }
                };
                let is_stored = match value.get("stored") {
                    Some(r) => match r.as_bool() {
                        Some(s) => s,
                        None => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                "field -> stored should be bool",
                            ));
                        }
                    },
                    None => false,
                };
                let is_fast = match value.get("fast") {
                    Some(r) => match r.as_bool() {
                        Some(s) => s,
                        None => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                "field -> fast should be bool",
                            ));
                        }
                    },
                    None => false,
                };
                let is_text = match value.get("text") {
                    Some(r) => match r.as_bool() {
                        Some(s) => s,
                        None => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                "field -> text should be bool",
                            ));
                        }
                    },
                    None => false,
                };
                let is_indexed = match value.get("indexed") {
                    Some(r) => match r.as_bool() {
                        Some(s) => s,
                        None => {
                            return Err(Error::new(
                                ErrorKind::Other,
                                "field -> indexed should be bool",
                            ));
                        }
                    },
                    None => false,
                };
                match field_type {
                    "u64" => {
                        let mut options = NumericOptions::default();
                        if is_stored {
                            options = options.set_stored();
                        }
                        if is_fast {
                            options = options.set_fast();
                        }
                        if is_indexed {
                            options = options.set_indexed();
                        }
                        schema_builder.add_u64_field(field_name, options);
                    }
                    "text" => {
                        let mut options = TextOptions::default();
                        if is_stored {
                            options = options.set_stored();
                        }
                        if is_fast {
                            options = options.set_fast(None);
                        }
                        if is_text {
                            options = options | TEXT
                        }
                        schema_builder.add_text_field(field_name, options);
                    }
                    "json" => {
                        let mut options = JsonObjectOptions::default();
                        if is_stored {
                            options = options.set_stored();
                        }
                        if is_fast {
                            options = options.set_fast(None);
                        }
                        if is_text {
                            options = options | TEXT
                        }
                        schema_builder.add_json_field(field_name, options);
                    }
                    "bool" => {
                        let mut options = NumericOptions::default();
                        if is_stored {
                            options = options.set_stored();
                        }
                        if is_fast {
                            options = options.set_fast();
                        }
                        if is_indexed {
                            options = options.set_indexed();
                        }
                        schema_builder.add_bool_field(field_name, options);
                    }
                    _ => {
                        return Err(Error::new(ErrorKind::Other, "unknown field type"));
                    }
                }
            }
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                "mappings has to contain properties",
            ));
        }
    } else {
        return Err(Error::new(
            ErrorKind::Other,
            "mappings has to contain properties",
        ));
    }
    let schema = schema_builder.build();
    Ok(schema)
}

fn create_index(name: &String, config: &ffi::IndexConfig) -> Result<ffi::Context, std::io::Error> {
    let mappings = match serde_json::from_str::<serde_json::Map<String, Value>>(&config.mappings) {
        Ok(r) => r,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Unable to parse mappings: {}", e),
            ));
        }
    };
    let schema = extract_schema(&mappings)?;
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
