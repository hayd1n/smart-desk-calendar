use embedded_svc::http::client::Client;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use esp_idf_svc::sys::EspError;

pub fn create_https_client() -> Result<Client<EspHttpConnection>, EspError> {
    let httpconnection = EspHttpConnection::new(&Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    Ok(Client::wrap(httpconnection))
}
