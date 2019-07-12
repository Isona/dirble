use crate::arg_parse::{GlobalOpts, LengthRange, LengthRanges};
use crate::output::{
    directory_name, print_response, sort_responses, startup_text,
};
use crate::request::RequestResponse;
use std::sync::Arc;

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
    let output =
        print_response(&rr, Arc::new(globalopts.clone()), false, false, false);
    assert_eq!(output, None, ".htaccess is not being hidden");
    // And check that they are not hidden otherwise
    globalopts.show_htaccess = true;
    let output = print_response(&rr, Arc::new(globalopts), false, false, false);
    assert_eq!(
        output,
        Some(String::from(
            "+ http://example.com/.htaccess (CODE:403|SIZE:234)"
        )),
        ".htaccess is not being hidden"
    );
}

#[test]
fn test_print_report() {}

#[test]
fn test_write_file() {}

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
    assert_eq!(
        num_test_cases,
        rr_vec.len(),
        "Length of test vector does not match expected number of cases"
    );

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
    let mut rr: RequestResponse = Default::default();
    // First case: rr is a directory ending with slash
    rr.url = "http://example.com/test/dir/".into();
    rr.is_directory = true;
    assert_eq!(
        directory_name(&rr),
        String::from("http://example.com/test/dir")
    );

    // Second case: rr is a directory not ending with slash
    rr.url = "http://example.com/test/dir".into();
    assert_eq!(
        directory_name(&rr),
        String::from("http://example.com/test/dir")
    );

    // Second case: rr is not a directory
    rr.is_directory = false;
    assert_eq!(directory_name(&rr), String::from("http://example.com/test"));
}

#[test]
fn test_create_files() {}

#[test]
fn test_generate_handle() {}

#[test]
fn test_startup_text() {
    let mut globalopts: GlobalOpts = Default::default();

    // Startup text should be None when the stdout is not going to a
    // terminal
    let text = startup_text(Arc::new(globalopts.clone()), &String::from("foo"));
    assert_eq!(text, None);

    // Startup text should have a simple value for default globalopts
    globalopts.is_terminal = true;
    let text = startup_text(Arc::new(globalopts.clone()), &String::from("foo"));
    // Version string changes with each build, so we need to get the
    // current value before validating the startup text. If the text is
    // not a Some(text) then the unwrap will panic and the test will
    // fail.
    let version = crate::arg_parse::get_version_string();
    let suffix = String::from(
        "\nDeveloped by Izzy Whistlecroft\nTargets: \nWordlist: foo\n\
         No Prefixes\nNo Extensions\nNo lengths hidden\n",
    );
    assert_eq!(text.unwrap(), format!("Dirble {}{}", version, suffix));

    // Set all of the optional parameters, output text should display
    // them.
    globalopts.hostnames =
        vec!["http://example.com".into(), "http://example.org".into()];
    globalopts.wordlist_files = Some(vec!["foo".into(), "bar".into()]);
    globalopts.prefixes = vec!["".into(), "~".into()];
    globalopts.extensions = vec!["".into(), ".txt".into(), ".com".into()];
    globalopts.length_blacklist = LengthRanges {
        ranges: vec![
            LengthRange {
                start: 2400,
                end: Some(3000),
            },
            LengthRange {
                start: 400,
                end: None,
            },
        ],
    };
    let text = startup_text(Arc::new(globalopts.clone()), &String::from("foo"));
    let suffix = String::from(
        "\nDeveloped by Izzy Whistlecroft\n\
         Targets: http://example.com http://example.org\n\
         Wordlists: foo bar\nPrefixes: ~\nExtensions: .txt .com\n\
         Hidden lengths: [400, 2400-3000]\n",
    );
    assert_eq!(text.unwrap(), format!("Dirble {}{}", version, suffix));
}
