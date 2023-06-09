use reqwest::blocking::RequestBuilder;

use crate::http::fetch::DataSourceResponse;

pub fn fetch(
    request: RequestBuilder,
    on_done: Box<dyn FnOnce(Result<DataSourceResponse, String>) + Send>,
) {
    std::thread::Builder::new()
        .name("ehttp".to_owned())
        .spawn(move || {
            let text = request
                .send()
                .expect("test")
                .text()
                .expect("unable to get text");

            on_done(Ok(DataSourceResponse { body: text }))
        })
        .expect("unable to spawn thread");
}
