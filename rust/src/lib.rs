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
        docId: u64,
    }

    #[namespace = "cxxtantivy"]
    extern "Rust" {
        type TantivyContext;
        fn add(input: &TextInput) -> bool;
        fn search(input: &SearchInput) -> SearchOutput;
    }
}

#[derive(Debug)]
pub struct TantivyContext {
}

fn add(input: &ffi::TextInput) -> bool {
    println!("ADDING: {}", input.data);
    true
}

fn search(input: &ffi::SearchInput) -> ffi::SearchOutput {
    println!("SEARCHING: query => {}", input.query);
    ffi::SearchOutput { docId: 7 }
}
