use std::sync::Arc;
use crate::request::RequestResponse;
use crate::arg_parse::GlobalOpts;
use crate::output::{
    print_response,
    sort_responses,
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
    let mut rr = RequestResponse {
        url: "http://example.com/".into(),
        code: 200,
        content_len: 200,
        is_directory: false,
        is_listable: false,
        redirect_url: "".into(),
        found_from_listable: false,
        parent_depth: 0,
    };

    // Generate a Vec of RequestResponses to sort
    let num_test_cases: usize = 4;
    let mut rr_vec: Vec<RequestResponse> = Vec::with_capacity(num_test_cases);

    rr_vec.push(rr.clone());

    rr.url = "http://example.com/two".into();
    rr_vec.push(rr.clone());

    rr.url = "http://example.com/one/three".into();
    rr_vec.push(rr.clone());

    rr.url = "http://example.com/one/".into();
    rr.is_directory = true;
    rr_vec.push(rr.clone());
    assert_eq!(num_test_cases, rr_vec.len(),
        "Length of test vector does not match expected number of cases");

    dbg!(&rr_vec);

    let sorted = sort_responses(rr_vec);

    dbg!(&sorted);

    let sorted_urls: Vec<String> = vec![
        "http://example.com/".into(),
        "http://example.com/two".into(),
        "http://example.com/one/".into(),
        "http://example.com/one/three".into(),
    ];

    assert_eq!(&sorted.len(), &sorted_urls.len());

    for pair in sorted.iter().zip(sorted_urls) {
        assert_eq!(pair.0.url, pair.1);
    }
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
