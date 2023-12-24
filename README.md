# cxxtantivy

## Notes/TODOs

* It's a bit tricky to introduce generic API, primarly becuase of speed -> measure first.
* Tantivy add_json_object accepts serde_json::map::Map<String, serde_json::value::Value>.
* Consider adding multiple workspaces under rust/ because multiple libraries could be added.
* Write unit / integration test to compare STRING vs JSON fiels search query syntax.
* Implement larger dataset generator.
* Add benchmark (add|retrieve simple|complex, filtering, aggregations).
* init -> create_index + add the ability to inject schema.
* Move includes to cxxtantivy/rust|cxx.hpp (consider mgcxxtantivy because of ffi).
* cxxtantivy::function but rust::Error -> unify.


