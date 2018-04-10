use exonum::api::Api;
use hyper::header::ContentType;
use hyper::mime::Mime;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use iron::status;
use prometheus;
use prometheus::{Encoder, TextEncoder};
use router::Router;

#[derive(Clone)]
pub struct MetricsApi {}

impl Api for MetricsApi {
    fn wire(&self, router: &mut Router) {
        let metrics = move |_: &mut Request| -> IronResult<Response> {
            let encoder = TextEncoder::new();
            let metric_families = prometheus::gather();
            let mut buffer = Vec::new();
            encoder.encode(&metric_families, &mut buffer).unwrap();

            let mut res = Response::with((status::Ok, String::from_utf8(buffer).unwrap()));
            res.headers
                .set(ContentType(encoder.format_type().parse::<Mime>().unwrap()));
            res.headers.set(AccessControlAllowOrigin::Any);

            Ok(res)
        };

        router.get("/metrics", metrics, "metrics");
    }
}
