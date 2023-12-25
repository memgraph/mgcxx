# cxxtantivy

## Notes/TODOs

* Move includes to cxxtantivy/rust|cxx.hpp (consider mgcxxtantivy because of ffi).
* init -> create_index + add the ability to inject schema.
* It's a bit tricky to introduce generic API, primarly becuase of speed -> measure first.
* Tantivy add_json_object accepts serde_json::map::Map<String, serde_json::value::Value>.
* Consider adding multiple workspaces under rust/ because multiple libraries could be added (memcxx as a repo).
* Write unit / integration test to compare STRING vs JSON fiels search query syntax.
* Implement larger dataset generator.
* Add benchmark (add|retrieve simple|complex, filtering, aggregations).
* cxxtantivy::function but rust::Error -> unify.

## Resources

* https://fulmicoton.com/posts/behold-tantivy-part2/
