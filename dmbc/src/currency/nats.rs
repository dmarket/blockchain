use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Once, ONCE_INIT};
use std::thread;
use std::time::{Duration, Instant};

use nats::Client;

use config;

pub fn publish(subject: String, msg: String) {
    let pipe = Pipe::get();
    pipe.as_ref()
        .map(|p| p.sender.send((subject, msg)).unwrap());
}

type PublishPair = (String, String);

static ONCE: Once = ONCE_INIT;
static mut PIPE: *mut Option<Pipe> = 0_usize as *mut _;

const DISCARD_MODE_SECONDS: u64 = 5;

enum Mode {
    Publish,
    Discard(Instant),
}

struct Pipe {
    thread: thread::JoinHandle<()>,
    sender: Sender<PublishPair>,
}

impl Pipe {
    fn get<'a>() -> &'a Option<Self> {
        unsafe {
            ONCE.call_once(|| {
                let pipe = if config::config().nats().enabled() {
                    let (sender, receiver) = mpsc::channel();
                    let thread = thread::Builder::new()
                        .name("NATS sender".to_string())
                        .spawn(|| Pipe::work(receiver))
                        .unwrap();
                    Some(Pipe { thread, sender })
                } else {
                    None
                };

                PIPE = Box::into_raw(Box::new(pipe));
            });
            &*PIPE
        }
    }

    fn work(receiver: Receiver<PublishPair>) {
        let mut client = match Client::new(config::config().nats().addresses()) {
            Ok(client) => client,
            Err(e) => {
                warn!("Error when creating NATS client: {}", e);
                return;
            }
        };

        let mut mode = Mode::Publish;
        let discard_duration = Duration::from_secs(DISCARD_MODE_SECONDS);

        let mut process_pair = |pair: PublishPair| match mode {
            Mode::Publish => match client.publish(&pair.0, pair.1.as_bytes()) {
                Ok(_) => info!("success published"),
                Err(e) => {
                    warn!("{:?}", e);
                    warn!("Discarding messages for {} seconds.", DISCARD_MODE_SECONDS);
                    mode = Mode::Discard(Instant::now());
                }
            },
            Mode::Discard(begin) if begin.elapsed() < discard_duration => (),
            Mode::Discard(_) => {
                info!("Accepting messages again.");
                mode = Mode::Publish;
            }
        };

        for pair in receiver {
            process_pair(pair);
        }
    }
}
