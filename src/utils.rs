use tonic::Request;

const CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CLIENT_NAME: &str = env!("CARGO_PKG_NAME");

/// Appends the api key metadata to the request, keyed by x-api-key.
pub(crate) fn append_api_key<T>(req: &mut Request<T>, key: &str) {
    req.metadata_mut().append("x-api-key", key.parse().unwrap());
}

/// Appends the client version metadata to the request, keyed by x-client-version.
pub(crate) fn append_client_version<T>(req: &mut Request<T>) {
    let client_full_name = format!("{}-rs/v{}", CLIENT_NAME, CLIENT_VERSION);
    req.metadata_mut()
        .append("x-client-version", client_full_name.parse().unwrap());
}

/// Appends the following metadata to the request:
/// - api-key: keyed x-api-key
/// - client version: keyed x-client-version
pub(crate) fn append_metadata<T>(req: &mut Request<T>, api_key: &str) {
    append_api_key(req, api_key);
    append_client_version(req);
}
