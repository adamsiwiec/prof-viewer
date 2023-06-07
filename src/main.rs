#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use legion_prof_viewer::http::queueclient::HTTPQueueDataSource;
use legion_prof_viewer::{console_log, logging::*};

use url::Url;

const DEFAULT_URL: &str = "http://127.0.0.1:8080";

fn main() {
    let loc: web_sys::Location = web_sys::window().unwrap().location();
    let href: String = loc.href().expect("Unable to get window URL");
    let browser_url = Url::parse(&href).expect("unable to parse url");

    console_log!("Browser URL: {}", href);
    let mut host: Option<Url> = None;
    browser_url.query_pairs().for_each(|(key, value)| {
        // check for host and port here
        if key == "url" {
            host = Some(Url::parse(&value).expect("Unable to parse url query parameter"));
        }
    });
    if host.is_none() {
        host = Some(Url::parse(DEFAULT_URL).expect("Unable to initialize default URL"));
    }

    log("Initializing Legion Profiler Viewer");
    // create queue
    legion_prof_viewer::app::start(Box::new(HTTPQueueDataSource::new(host.unwrap())), None);
}
