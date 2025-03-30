// This file is part of Dirble - https://www.github.com/nccgroup/dirble
// Copyright (C) 2019 Izzy Whistlecroft <Izzy(dot)Whistlecroft(at)nccgroup(dot)com>
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

use crate::arg_parse::GlobalOpts;
use std::io::{Read, Write};
use tempfile::NamedTempFile;

#[test]
fn basic_test() {
    let port = crate::test_server::launch();
    let wordlist = crate::test_server::PATHS
        .keys()
        .copied()
        .chain(std::iter::once("notfound"))
        .collect::<Vec<_>>();

    let mut wordlist_file = NamedTempFile::new().unwrap();
    for word in wordlist {
        writeln!(&mut wordlist_file, "{word}").unwrap();
    }

    let mut output_file = NamedTempFile::new().unwrap();

    let args = GlobalOpts {
        hostnames: vec![format!("http://localhost:{port}").parse().unwrap()],
        wordlist_files: Some(vec![wordlist_file.path().display().to_string()]),
        output_file: Some(output_file.path().display().to_string()),
        ..GlobalOpts::default()
    };

    crate::dirble_main(args);

    let mut output = String::new();
    output_file.read_to_string(&mut output).unwrap();

    insta::with_settings!({
        filters => vec![
            ("localhost:[0-9]{1,5}","localhost"),
        ]}, {
            insta::assert_snapshot!(output);
    });
}
