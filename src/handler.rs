use crate::emulator::{CpuError, System};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

static TEST_CODE: &'static [u8; 0x10000] = include_bytes!("color.hex");

/* #region Commands */
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cmd {
    Step,
    Run,
    Stop,
    Get(GetType),
    Reset,
}
impl std::convert::From<&str> for Cmd {
    fn from(text: &str) -> Self {
        match text {
            "Step" => Self::Step,
            "Run" => Self::Run,
            "Stop" => Self::Stop,
            "Reset" => Self::Reset,
            _ => panic!("???"),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GetType {
    Range(usize, usize),
    Value(usize),
    Flags,
}
/* #endregion */

pub struct ThreadedEmulator {
    pub tcmd: mpsc::Sender<Cmd>,
    pub rdata: glib::Receiver<Vec<u8>>,
    pub system: Arc<Mutex<System>>,
    pub thread: thread::JoinHandle<Result<(), CpuError>>,
}
impl ThreadedEmulator {
    pub fn new() -> Self {
        let (tcmd, rcmd) = mpsc::channel::<Cmd>();
        let (tdata, rdata) = glib::MainContext::channel(glib::source::Priority::default());
        let system = Arc::from(Mutex::from(System::new()));
        let thread = {
            let system = system.clone();
            thread::spawn(move || Self::thread(rcmd, tdata, system))
        };
        Self {
            tcmd,
            rdata,
            system,
            thread,
        }
    }

    fn thread(
        rcmd: mpsc::Receiver<Cmd>,
        tdata: glib::Sender<Vec<u8>>,
        system: Arc<Mutex<System>>,
    ) -> Result<(), CpuError> {
        loop {
            if let Ok(cmd) = rcmd.recv() {
                let mut system = system.lock().unwrap_or_else(|e| {
                    panic!("Error acquiring lock for the system. Error: {}", e)
                });
                println!("Cmd: {:?}", cmd);
                match cmd {
                    Cmd::Step => system.step()?,
                    Cmd::Reset => {
                        system.restart();
                        system.ram = *TEST_CODE;
                    }
                    Cmd::Run => loop {
                        if let Err(e) = rcmd.recv_timeout(std::time::Duration::from_millis(1)) {
                            if e == std::sync::mpsc::RecvTimeoutError::Timeout {
                                system.step()?;
                            } else {
                                panic!("Controller mpsc disconnected: {}", e)
                            }
                        } else {
                            break;
                        }
                    },
                    Cmd::Get(what) => match what {
                        GetType::Flags => {}
                        GetType::Range(start, end) => {
                            let data = &system.ram[start..end];
                            tdata
                                .send(Vec::from(data))
                                .expect("Couldn't send requested value");
                        }
                        GetType::Value(addr) => {
                            tdata
                                .send(vec![system.ram[addr]])
                                .expect("Couldn't send requested value");
                        }
                    },
                    _ => {}
                };
            }
        }
    }
}
