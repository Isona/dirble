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
