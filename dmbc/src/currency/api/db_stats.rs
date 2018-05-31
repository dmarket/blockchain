use exonum::api::Api;
use exonum_rocksdb::Options;
use hyper::header::ContentType;
use iron::headers::AccessControlAllowOrigin;
use iron::prelude::*;
use iron::status;
use router::Router;

pub struct DbStatsApi {}

impl Api for DbStatsApi {
    fn wire(&self, router: &mut Router) {
        let stats = move |_: &mut Request| -> IronResult<Response> {
            let stats = Options::default().get_statistics();
            let res = match stats {
                Some(stat_string) => {
                    let mut res = Response::with((status::Ok, stat_string));
                    res.headers.set(ContentType::plaintext());
                    res.headers.set(AccessControlAllowOrigin::Any);
                    res
                }
                None => {
                    let mut res = Response::with(
                        (status::ServiceUnavailable,
                         "")
                    );
                    res.headers.set(ContentType::plaintext());
                    res.headers.set(AccessControlAllowOrigin::Any);
                    res
                }
            };
            Ok(res)
        };

        router.get("/db_stats", stats, "stats");
    }
}
