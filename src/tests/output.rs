use std::sync::Arc;
use crate::request::RequestResponse;
use crate::arg_parse::GlobalOpts;
use crate::output::{
    print_response,
};

#[test]
fn test_print_response() {
    let rr = RequestResponse {
        url: "http://example.com".into(),
        code: 200,
        content_len: 234,
        is_directory: false,
        is_listable: false,
        redirect_url: Default::default(),
        found_from_listable: false,
        parent_depth: 0,
    };
    let globalopts: GlobalOpts = Default::default();
    let globalopts = Arc::new(globalopts);

    let output = print_response(&rr, globalopts.clone(), false, false, false);

    assert!(output != None);
}
