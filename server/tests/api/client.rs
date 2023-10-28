use crate::{helper, server, test_utils};

#[derive(Debug, Clone)]
pub struct Client {
    client: hyper::Client<hyper::client::HttpConnector>,
    uri: std::sync::Arc<String>,
}

impl From<&server::Server> for Client {
    fn from(server: &server::Server) -> Self {
        Self {
            client: hyper::Client::new(),
            uri: server.uri(),
        }
    }
}

impl Client {
    pub async fn get(&self, path_and_query: impl AsRef<str>) -> hyper::Response<hyper::Body> {
        let request = self.get_request(path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn get_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = self.get_request(path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn post(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Response<hyper::Body> {
        let request = self.post_request(path_and_query, body);
        self.client.request(request).await.unwrap()
    }

    pub async fn put(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Response<hyper::Body> {
        let request = self.put_request(path_and_query, body);
        self.client.request(request).await.unwrap()
    }

    pub async fn put_with(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = self.put_request(path_and_query, body);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn delete(&self, path_and_query: impl AsRef<str>) -> hyper::Response<hyper::Body> {
        let request = self.delete_request(path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn delete_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = self.delete_request(path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn head(&self, path_and_query: impl AsRef<str>) -> hyper::Response<hyper::Body> {
        let request = self.head_request(path_and_query);
        self.client.request(request).await.unwrap()
    }

    pub async fn head_with(
        &self,
        path_and_query: impl AsRef<str>,
        f: impl Fn(&mut hyper::Request<hyper::Body>),
    ) -> hyper::Response<hyper::Body> {
        let mut request = self.head_request(path_and_query);
        f(&mut request);
        self.client.request(request).await.unwrap()
    }

    pub async fn last_modified(&self, path_and_query: impl AsRef<str>) -> u64 {
        let request = self.head_request(path_and_query);
        let response = self.client.request(request).await.unwrap();
        helper::extract_last_modified(&response).unwrap()
    }
}

impl Client {
    fn request(&self, path_and_query: impl AsRef<str>) -> hyper::http::request::Builder {
        let uri = hyper::Uri::builder()
            .scheme("http")
            .authority(self.uri.as_str())
            .path_and_query(path_and_query.as_ref())
            .build()
            .unwrap();

        hyper::Request::builder()
            .uri(uri)
            .header(helper::USER_HEADER, test_utils::USER)
    }

    fn get_request(&self, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::Body> {
        self.request(path_and_query)
            .method(hyper::Method::GET)
            .body(hyper::Body::empty())
            .unwrap()
    }

    fn post_request(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Request<hyper::Body> {
        self.request(path_and_query)
            .method(hyper::Method::POST)
            .body(body.into())
            .unwrap()
    }

    fn put_request(
        &self,
        path_and_query: impl AsRef<str>,
        body: impl Into<hyper::Body>,
    ) -> hyper::Request<hyper::Body> {
        self.request(path_and_query)
            .method(hyper::Method::PUT)
            .header(hyper::header::IF_UNMODIFIED_SINCE, millis_in_future())
            .body(body.into())
            .unwrap()
    }

    fn delete_request(&self, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::Body> {
        self.request(path_and_query)
            .method(hyper::Method::DELETE)
            .header(hyper::header::IF_UNMODIFIED_SINCE, millis_in_future())
            .body(hyper::Body::empty())
            .unwrap()
    }

    fn head_request(&self, path_and_query: impl AsRef<str>) -> hyper::Request<hyper::Body> {
        self.request(path_and_query)
            .method(hyper::Method::HEAD)
            .body(hyper::Body::empty())
            .unwrap()
    }
}

fn millis_in_future() -> hyper::http::HeaderValue {
    hyper::header::HeaderValue::from_str(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .saturating_add(std::time::Duration::from_secs(10))
            .as_millis()
            .to_string()
            .as_str(),
    )
    .unwrap()
}
