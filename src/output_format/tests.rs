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
fn check_json_format() {
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
