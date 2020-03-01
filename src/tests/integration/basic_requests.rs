// This file is part of Dirble - https://www.github.com/nccgroup/dirble
// Copyright (C) 2019
//  * Izzy Whistlecroft <Izzy(dot)Whistlecroft(at)nccgroup(dot)com>
//  * David Young <David(dot)Young(at)nccgroup(dot)com>
// Released as open source by NCC Group Plc - https://www.nccgroup.com/
//
// Dirble is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Dirble is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Dirble.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::Arc;
use url::Url;

use super::mock_server;
use crate::arg_parse::GlobalOpts;
use crate::request::{self, RequestResponse};

#[test]
fn test_200_response() {
    mock_server();

    let global_opts: GlobalOpts = Default::default();
    let global_opts = Arc::new(global_opts);
    let mut easy = request::generate_easy(&global_opts);

    let url = Url::parse(super::URL).unwrap();

    let rr = request::make_request(&mut easy, url);

    println!("{:?}", rr);

    assert_eq!(
        rr,
        RequestResponse {
            url: Url::parse("http://[::1]:3000/").unwrap(),
            code: 200,
            content_len: 2,
            is_directory: false,
            is_listable: false,
            redirect_url: "".into(),
            found_from_listable: false,
            parent_index: 0,
            parent_depth: 0,
        },
        "Response not recognised :(",
    );
}

#[test]
fn test_301_response() {
    mock_server();

    let global_opts: GlobalOpts = Default::default();
    let global_opts = Arc::new(global_opts);
    let mut easy = request::generate_easy(&global_opts);

    let url = Url::parse(super::URL).unwrap().join("301.html").unwrap();

    let rr = request::make_request(&mut easy, url);

    println!("{:?}", rr);

    assert_eq!(
        rr,
        RequestResponse {
            url: Url::parse("http://[::1]:3000/301.html").unwrap(),
            code: 301,
            content_len: 8,
            is_directory: false,
            is_listable: false,
            redirect_url: "http://[::1]:3000/301-target.html".into(),
            found_from_listable: false,
            parent_index: 0,
            parent_depth: 0,
        },
        "Response not recognised :(",
    );
}
