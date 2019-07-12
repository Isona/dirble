use crate::arg_parse::{GlobalOpts, LengthRange, LengthRanges};
use crate::request::RequestResponse;
use crate::request_thread::should_send_response;
use crate::validator_thread::TargetValidator;

#[test]
fn test_should_send_response() {
    let mut globalopts: GlobalOpts = Default::default();
    let mut rr: RequestResponse = Default::default();

    // Response code is in blacklist -> false
    globalopts.whitelist = false;
    globalopts.code_list = vec![200, 201];
    rr.code = 200;
    assert_eq!(
        should_send_response(&globalopts, &rr, &None),
        false,
        "Code in blacklist failed"
    );

    // Response code is not in blacklist -> true
    rr.code = 300;
    assert_eq!(
        should_send_response(&globalopts, &rr, &None),
        true,
        "Code not in blacklist failed"
    );

    // Response code is in whitelist -> true
    globalopts.whitelist = true;
    rr.code = 200;
    assert_eq!(
        should_send_response(&globalopts, &rr, &None),
        true,
        "Code in whitelist failed"
    );

    // Response code is not in whitelist -> false
    rr.code = 301;
    assert_eq!(
        should_send_response(&globalopts, &rr, &None),
        false,
        "Code not in whitelist failed"
    );

    // Response matches Not Found condition -> false
    globalopts.whitelist = false;
    let val = TargetValidator::new(301, None, None, None, None);
    assert_eq!(
        should_send_response(&globalopts, &rr, &Some(val)),
        false,
        "Not Found response failed"
    );

    // Response length exactly matches a blacklist item -> false
    rr.content_len = 500;
    globalopts.length_blacklist = LengthRanges {
        ranges: vec![
            LengthRange {
                start: 5000,
                end: Some(6000),
            },
            LengthRange {
                start: 500,
                end: None,
            },
        ],
    };
    assert_eq!(
        should_send_response(&globalopts, &rr, &None),
        false,
        "Length matches blacklist failed"
    );

    // Response length is within a blacklist range -> false
    rr.content_len = 5300;
    assert_eq!(
        should_send_response(&globalopts, &rr, &None),
        false,
        "Length within blacklist range failed"
    );

    // Response length_is outside of the blacklist ranges -> true
    rr.content_len = 5;
    assert_eq!(
        should_send_response(&globalopts, &rr, &None),
        true,
        "Length outside blacklist range failed"
    );
}
