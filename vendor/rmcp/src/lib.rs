#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![forbid(unsafe_code)]
// include! makes upstream's exported macros appear macro-expanded; preserve
// the SDK's existing crate-qualified macro imports without rewriting its code.
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
#![doc = include_str!(concat!(env!("OUT_DIR"), "/rmcp-3.6.0/README.md"))]

// The build wrapper verifies and unpacks the official SDK, applies the declared
// codec hunks, and gives its root modules explicit paths into that source tree.
include!(concat!(env!("OUT_DIR"), "/rmcp-lib.rs"));
