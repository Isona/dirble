use std::sync::Arc;
use crate::request::RequestResponse;
use crate::arg_parse::GlobalOpts;
use crate::output::{
    print_response,
};

#[test]
fn test_print_response() {
    // Only the hidden and not hidden parts of output::print_response
    // need to be tested - the rest is covered by the testing in
    // tests/output_format.rs.
    let rr = RequestResponse {
        url: "http://example.com/.htaccess".into(),
        code: 403,
        content_len: 234,
        is_directory: false,
        is_listable: false,
        redirect_url: Default::default(),
        found_from_listable: false,
        parent_depth: 0,
    };
    let mut globalopts: GlobalOpts = Default::default();

    // Verify that htaccess files are hidden when the option is set in
    // globalopts
    globalopts.show_htaccess = false;
    let output = print_response(
        &rr, Arc::new(globalopts.clone()), false, false, false);
    assert_eq!(output,
               None,
               ".htaccess is not being hidden");
    // And check that they are not hidden otherwise
    globalopts.show_htaccess = true;
    let output = print_response(&rr, Arc::new(globalopts), false, false, false);
    assert_eq!(output,
              Some(String::from(
                      "+ http://example.com/.htaccess (CODE:403|SIZE:234)")),
              ".htaccess is not being hidden");
}

#[test]
fn test_print_report() {
}

#[test]
fn test_write_file() {
}

#[test]
fn test_sort_responses() {
}

#[test]
fn test_directory_name() {
}

#[test]
fn test_create_files() {
}

#[test]
fn test_generate_handle() {
}

#[test]
fn test_startup_text() {
}
