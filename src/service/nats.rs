use std::sync::{Once, ONCE_INIT};
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

use nats::Client;

use config;

pub fn publish(subject: String, msg: String) {
    let pipe = Pipe::get();
    pipe.sender.send((subject, msg)).unwrap();
}

type PublishPair = (String, String);

static ONCE: Once = ONCE_INIT;
static mut PIPE: *mut Pipe = 0_usize as *mut _;

struct Pipe {
    thread: thread::JoinHandle<()>,
    sender: Sender<PublishPair>,
}

impl Pipe {
    fn get<'a>() -> &'a Self {
        unsafe {
            ONCE.call_once(||{
                let (sender, receiver) = mpsc::channel();
                let thread = thread::Builder::new()
                    .name("NATS sender".to_string())
                    .spawn(|| Pipe::work(receiver))
                    .unwrap();
                PIPE = Box::into_raw(Box::new(Pipe{ thread, sender }));
            });
            &*PIPE
        }
    }

    fn work(receiver: Receiver<PublishPair>) {
        match Client::new(config::config().nats().addresses()) {
            Ok(mut client) => {
                for pair in receiver {
                    match client.publish(&pair.0, pair.1.as_bytes()) {
                        Ok(_) => println!("success published"),
                        Err(e) => println!("{:?}", e),
                    }
                }
            }
            Err(e) => println!("NATS server error {:?}", e),
        }
    }
}

