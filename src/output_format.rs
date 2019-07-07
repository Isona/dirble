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

use crate::request::RequestResponse;
use colored::*;
use serde_json;
use simple_xml_serialize::XMLElement;

#[cfg(test)]
mod tests;

#[inline]
pub fn output_indentation(response: &RequestResponse, print_newlines: bool, indentation: bool) -> String {
    let mut output: String;

    if response.is_directory && print_newlines {
        output = String::from("\n");
    }
    else {
        output = String::from("");
    }

    if !indentation { return output }

    let mut depth = response.url.matches("/").count() as i32;

    if response.url.ends_with("/") {
        depth -= 1;
    }

    depth -= response.parent_depth as i32; 
    depth -= 1;
    
    if depth <= 0 {
        return output
    }

    for _ in 0..depth {
        output += "  ";
    }

    output
}

#[inline]
pub fn output_letter(response: &RequestResponse) -> String {
    if response.is_directory && response.is_listable { "L ".bold().to_string() }
    else if response.is_directory { String:: from("D ") }
    else if response.found_from_listable { String::from("~ ") }
    else { String::from("+ ") }
}

#[inline]
pub fn output_url(response: &RequestResponse) -> String {
    format!("{} ", response.url)
}

#[inline]
pub fn output_suffix(response: &RequestResponse, color: bool) -> String {
    if response.found_from_listable { return String::from("(SCRAPED)") }

    let mut code_string:String = format!{"{}", response.code};
    if color {
        code_string = match response.code {
            200..=299 => { code_string.green().to_string() }
            300..=399 => { code_string.cyan().to_string() }
            400..=499 => { code_string.red().to_string() }
            500..=599 => { code_string.yellow().to_string() }
            _ => { code_string }
        }
    }

    match response.code {
        301 | 302 => {
            format!("(CODE:{}|SIZE:{:#?}|DEST:{})", 
                code_string, response.content_len, response.redirect_url)
        }
        _ => {
            format!("(CODE:{}|SIZE:{:#?})", code_string, response.content_len)
        }
    }
}

#[inline]
pub fn output_xml(response: &RequestResponse) -> String {
    format!("{}\n", XMLElement::from(response).to_string())
}

#[inline]
pub fn output_json(response: &RequestResponse) -> String {
    serde_json::to_string(response).unwrap()
}
