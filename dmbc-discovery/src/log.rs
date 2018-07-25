use slog::{Logger, Drain};
use slog_async;
use slog_term;

lazy_static!{
    pub static ref ROOT: Logger = {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        Logger::root(drain, o!())
    };

    pub static ref SERVER: Logger = Logger::new(&ROOT, o!("subsystem" => "SERVER"));
    pub static ref KEEPER: Logger = Logger::new(&ROOT, o!("subsystem" => "KEEPER"));
}

