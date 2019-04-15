#[test]
fn check_output_indentation() {
    //   super::output_indentation produces a number of spaces based on
    // the parent_depth field of the RR and the number of slashes in
    // the url.
    //   Types of output:
    //     * Indentation disabled: empty String
    //     * Two spaces for each level below the parent directory the
    //       URL is
    //     * A preceding newline is added if print_newlines is set and
    //       the RR is a directory

    // Indentation disabled
    let mut req_response = generate_request_response();
    assert_eq!(
        super::output_indentation(&req_response, false, false),
        "",
        "Disabling of indentation does not stop the indentation");

    // Preceding newline, indentation disabled to force early return
    req_response.is_directory = true;
    assert_eq!(
        super::output_indentation(&req_response, true, false),
        "\n",
        "Newline is not printed");

    // Default indentation of zero spaces for base URL
    assert_eq!(
        super::output_indentation(&req_response, false, true),
        "",
        "Default empty indentation not returned");

    // Indentation of four spaces for file three levels deep
    req_response.is_directory = false;
    req_response.url = "http://example.com/a/test/directory".into();
    assert_eq!(
        super::output_indentation(&req_response, false, true),
        "    ", // four spaces
        "Indentation of nested directories incorrect");

    // Same scenario, but with a trailing slash
    req_response.url = "http://example.com/a/test/directory/".into();
    assert_eq!(
        super::output_indentation(&req_response, false, true),
        "    ", // four spaces
        "Trailing slash is not taken into account");
}

#[test]
fn check_output_letter () {
    // Check that:
    // * directory && listable -> L
    // * directory && !listable -> D
    // * found from listable -> ~
    // * otherwise -> +
    // (all with trailing space)
    let mut req_response = generate_request_response();
    req_response.is_directory = true;
    req_response.is_listable = true;
    assert_eq!(
        super::output_letter(&req_response),
        "L ",
        "Listable directory prefix incorrect");

    req_response.is_listable = false;
    assert_eq!(
        super::output_letter(&req_response),
        "D ",
        "Directory prefix incorrect");

    req_response.is_directory = false;
    req_response.found_from_listable = true;
    assert_eq!(
        super::output_letter(&req_response),
        "~ ",
        "Found from listable prefix incorrect");

    req_response.found_from_listable = false;
    assert_eq!(
        super::output_letter(&req_response),
        "+ ",
        "Regular file prefix incorrect");
}

#[test]
fn check_output_url() {
    // output_url simply returns the url, but with a space at the end
    let req_response = generate_request_response();
    assert_eq!(
        super::output_url(&req_response),
        format!("{} ", req_response.url),
        "Output URL formatted incorrectly");
}

#[test]
fn check_output_suffix() {
    // output_suffix takes a RR and returns a string of the format
    // (CODE:{}|SIZE{}), where the code is coloured appropriately.
    let mut req_response = generate_request_response();
    req_response.content_len = 456;
    req_response.code = 201;
    assert_eq!(
        super::output_suffix(&req_response, true),
        "(CODE:\u{1b}[32m201\u{1b}[0m|SIZE:456)",
        "Output suffix for code 201 invalid");

    req_response.code = 304;
    assert_eq!(
        super::output_suffix(&req_response, true),
        "(CODE:\u{1b}[36m304\u{1b}[0m|SIZE:456)",
        "Output suffix for code 304 invalid");

    // Test that the redirect URL is included
    req_response.code = 301;
    req_response.redirect_url = "https://nccgroup.com".into();
    assert_eq!(
        super::output_suffix(&req_response, true),
        "(CODE:\u{1b}[36m301\u{1b}[0m|SIZE:456|DEST:https://nccgroup.com)",
        "Output suffix for code 301 invalid");

    req_response.code = 451;
    assert_eq!(
        super::output_suffix(&req_response, true),
        "(CODE:\u{1b}[31m451\u{1b}[0m|SIZE:456)",
        "Output suffix for code 451 invalid");

    req_response.code = 503;
    assert_eq!(
        super::output_suffix(&req_response, true),
        "(CODE:\u{1b}[33m503\u{1b}[0m|SIZE:456)",
        "Output suffix for code 503 invalid");

    // Check that turning off colours also works
    assert_eq!(
        super::output_suffix(&req_response, false),
        "(CODE:503|SIZE:456)",
        "Disabling colours hasn't worked properly");
}

#[test]
fn check_json_format() {
    // This doesn't use the generate_request_response function because
    // the defaults may change but the expected JSON output is
    // hardcoded.
    let req_response = super::RequestResponse {
        url: "http://example.com".into(),
        code: 200,
        content_len: 350,
        is_directory: false,
        is_listable: true,
        found_from_listable: false,
        redirect_url: "https://example.org".into(),
        parent_depth: 0
    };
    let json = super::output_json(&req_response);

    assert_eq!(
        json,
        "{\
        \"url\": \"http://example.com\", \
            \"code\": 200, \
            \"size\": 350, \
            \"is_directory\": false, \
            \"is_listable\": true, \
            \"found_from_listable\": false, \
            \"redirect_url\": \"https://example.org\"\
            }\
            ",
            "JSON output appears invalid!");
}

#[inline]
fn generate_request_response() -> super::RequestResponse {
    // Generate a RequestResponse object with sane default settings to
    // simplify the testing routines.
    super::RequestResponse {
        url: "http://example.com".into(),
        code: 200,
        content_len: 350,
        is_directory: false,
        is_listable: false,
        found_from_listable: false,
        redirect_url: "https://example.org".into(),
        parent_depth: 2 // Depth is number of slashes, 2 for http://
    }
}
