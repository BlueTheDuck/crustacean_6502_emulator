mod error;
use error::ProgErr;

use cairo;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;

mod emulator;
use emulator::Emulator;
mod graphic;
use graphic::{Color, Image};

#[derive(Copy, Clone, Debug, PartialEq)]
enum Cmd {
    Step,
    Run,
    Stop,
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

pub fn init(app: &gtk::Application) {
    let img = Image::new();
    let (img_width, img_height) = (img.width, img.height);
    use std::sync::{Arc, Mutex};
    let img_m = Arc::from(Mutex::from(img));

    let (tcmd, rcmd) = std::sync::mpsc::channel::<Cmd>();
    let (tdata, rdata) = glib::MainContext::channel(glib::source::Priority::default());
    let emulator = Arc::from(Mutex::from(Emulator::new()));
    {
        let emulator_clone = emulator.clone();
        std::thread::spawn(move || {
            let emulator = emulator_clone;
            loop {
                if let Ok(cmd) = rcmd.recv() {
                    let mut emulator = match emulator.try_lock() {
                        Err(_) => continue,
                        Ok(v) => v,
                    };
                    match cmd {
                        Cmd::Step => {
                            if let Err(e) = emulator.step() {
                                println!("{:#?}", e);
                            }
                        }
                        Cmd::Reset => {
                            emulator.restart();
                            emulator.ram = *include_bytes!("color.hex");
                        }
                        Cmd::Run => loop {
                            if let Ok(cmd) = rcmd.recv_timeout(std::time::Duration::from_millis(1))
                            {
                                break;
                            } else {
                                emulator.step();
                                println!("Tick {}", emulator.cycles);
                                let page_02 = Vec::from(&emulator.ram[0x200..0x300]);
                                tdata.send(page_02);
                            }
                        },
                        _ => {}
                    };
                    println!("Tick {}", emulator.cycles);
                    let page_02 = Vec::from(&emulator.ram[0x200..0x300]);
                    tdata.send(page_02);
                }
            }
        });
    }
    let builder: Builder = Builder::new_from_string(include_str!("ui.glade"));
    let window: gtk::ApplicationWindow = builder.get_object("Window").unwrap();
    let da: gtk::DrawingArea = builder.get_object("Display").unwrap();

    {
        da.set_size_request(img_width as i32, img_height as i32);
        let img_m = img_m.clone();
        da.connect_draw(move |_: &gtk::DrawingArea, ctx: &cairo::Context| {
            let img: Result<_, _> = img_m.try_lock();
            if let Ok(img) = img {
                println!("Redrawing!");
                img.draw(&ctx);
            }
            glib::signal::Inhibit(false)
        });
    }

    for widget_name in &["Step", "Reset", "Run"] {
        let tcmd_clone = tcmd.clone();
        let widget: gtk::Button = builder.get_object(widget_name).expect("Not found");
        widget.connect_clicked(move |s: &gtk::Button| {
            let name = s.get_widget_name().unwrap();
            let name = name.as_str();
            println!("Sending from {}", name);
            let cmd = Cmd::from(name);
            tcmd_clone.send(cmd).expect("Couldn't send cmd");
        });
    }

    let registers: gtk::Label = builder.get_object("Registers").unwrap();
    let registers = Mutex::from(registers);

    {
        let emulator = emulator.clone();
        let img_m = img_m.clone();
        rdata.attach(None, move |data: Vec<_>| {
            let img: Result<_, _> = img_m.try_lock();
            println!("Received page");
            for line in data.chunks(16) {
                println!(
                    "{}",
                    line.iter()
                        .map(|h| format!("{:02X}", h))
                        .collect::<String>()
                )
            }
            if let Ok(mut img) = img {
                println!("Updating image");
                let (w, h) = (img.width, img.height);
                let pixels: &mut [Color] = &mut img.pixels;
                let palette = [
                    Color::from(0x00_00_00),
                    Color::from(0xFF_00_00),
                    Color::from(0x00_FF_00),
                    Color::from(0xFF_FF_00),
                    Color::from(0x00_00_FF),
                    Color::from(0xFF_00_FF),
                    Color::from(0x00_FF_FF),
                    Color::from(0xFF_FF_FF),
                ];
                for x in 0..w {
                    for y in 0..h {
                        let i = y * w + x;
                        pixels[i] = palette[data[i] as usize & 0b111];
                    }
                }
            }
            if let Ok(registers) = registers.try_lock() {
                if let Ok(emulator) = emulator.try_lock() {
                    registers.set_text(&format!("{:#?}", emulator.cpu));
                }
            }
            da.queue_draw();
            glib::Continue(true)
        });
    }

    window.set_application(Some(app));
    window.show_all();
}

fn main() -> Result<(), ProgErr> {
    let app: gtk::Application =
        gtk::Application::new(Some("com.ducklings_corp.emulator"), Default::default())?;
    app.connect_activate(move |app| init(app));
    app.run(&std::env::args().collect::<Vec<_>>());
    Ok(())
}
