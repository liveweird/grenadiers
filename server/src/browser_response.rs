use axum::{
    body::Body,
    http::{
        HeaderName, HeaderValue, Request, Response,
        header::{CACHE_CONTROL, CONTENT_TYPE},
    },
    middleware::Next,
};

const X_CONTENT_TYPE_OPTIONS: HeaderName = HeaderName::from_static("x-content-type-options");
const X_FRAME_OPTIONS: HeaderName = HeaderName::from_static("x-frame-options");
const REFERRER_POLICY: HeaderName = HeaderName::from_static("referrer-policy");

const NO_CACHE: HeaderValue = HeaderValue::from_static("no-cache");
const IMMUTABLE: HeaderValue = HeaderValue::from_static("public, max-age=31536000, immutable");

pub(crate) async fn headers_and_cache(request: Request<Body>, next: Next) -> Response<Body> {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));

    if !headers.contains_key(CACHE_CONTROL) {
        let html = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/html"));
        let cache = if response.status().is_success() && !html && is_content_hashed_asset(&path) {
            IMMUTABLE
        } else {
            NO_CACHE
        };
        response.headers_mut().insert(CACHE_CONTROL, cache);
    }
    response
}

fn is_content_hashed_asset(path: &str) -> bool {
    let Some(filename) = path
        .strip_prefix("/assets/")
        .and_then(|path| path.rsplit('/').next())
    else {
        return false;
    };
    let Some((stem, extension)) = filename.rsplit_once('.') else {
        return false;
    };
    const VITE_HASH_BYTES: usize = 8;
    let bytes = stem.as_bytes();
    let Some(delimiter_index) = bytes.len().checked_sub(VITE_HASH_BYTES + 1) else {
        return false;
    };
    let hash = &bytes[delimiter_index + 1..];
    delimiter_index > 0
        && bytes[delimiter_index] == b'-'
        && !extension.is_empty()
        && hash
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::is_content_hashed_asset;

    #[test]
    fn recognizes_only_bounded_vite_style_asset_hashes() {
        assert!(is_content_hashed_asset("/assets/index-C4t9aB2_.js"));
        assert!(is_content_hashed_asset("/assets/Geometry-DKID-A9Y.js"));
        assert!(is_content_hashed_asset(
            "/assets/getTextureBatchBindGroup-Bv-Zrb3q.js"
        ));
        assert!(is_content_hashed_asset("/assets/terrainPixel-D2laAj-4.js"));
        assert!(!is_content_hashed_asset("/index-C4t9aB2_.js"));
        assert!(!is_content_hashed_asset("/assets/index.js"));
        assert!(!is_content_hashed_asset("/assets/branding-main-logo.svg"));
        assert!(!is_content_hashed_asset("/assets/app.js"));
        assert!(!is_content_hashed_asset(
            "/assets/icons/map-0123456789abcdef.svg"
        ));
        assert!(!is_content_hashed_asset("/assets/index-v1.2.3.js"));
        assert!(!is_content_hashed_asset("/assets/index-1234567.js"));
    }
}
