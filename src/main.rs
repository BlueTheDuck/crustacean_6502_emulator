mod error;
use error::ProgErr;

use cairo;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;

mod sixty_five;
use sixty_five::Emulator;
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

pub fn init(app: &gtk::Application) -> Result<(), ProgErr> {
    let img = Image::new();
    let (img_width, img_height) = (img.width, img.height);
    use std::sync::{Arc, Mutex};
    let img_m = Arc::from(Mutex::from(img));
    let palette = Arc::from(Mutex::from([Color::default(); 16]));

    // Handle emulator CMDs
    let (tcmd, rcmd) = std::sync::mpsc::channel::<Cmd>();
    let (tdata, rdata) = glib::MainContext::channel(glib::source::Priority::default());
    let emulator = Arc::from(Mutex::from(Emulator::new()));
    {
        let emulator = emulator.clone();
        std::thread::spawn(move || loop {
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
                        if let Ok(_) = rcmd.recv_timeout(std::time::Duration::from_millis(1)) {
                            break;
                        } else {
                            if let Err(e) = emulator.step() {
                                println!("Emulator error: {:?}", e);
                                break;
                            }
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
        });
    }

    let builder: Builder = Builder::new_from_string(include_str!("ui.glade"));
    let window: gtk::ApplicationWindow = builder.get_object("Window").unwrap();
    let drawing_area: gtk::DrawingArea = builder.get_object("Display").unwrap();
    let drawing_area = Arc::from(drawing_area);
    {
        drawing_area.set_size_request(img_width as i32, img_height as i32);
        let img_m = img_m.clone();
        drawing_area.connect_draw(
            move |drawing_area: &gtk::DrawingArea, ctx: &cairo::Context| {
                let widget_width = drawing_area.get_allocated_width();
                let widget_height = drawing_area.get_allocated_height();
                let img: Result<_, _> = img_m.try_lock();
                if let Ok(img) = img {
                    img.draw(&ctx, (widget_width, widget_height));
                }
                glib::signal::Inhibit(false)
            },
        );
    }

    for widget_name in &["Step", "Reset", "Run", "Stop"] {
        let tcmd_clone = tcmd.clone();
        let widget: gtk::Button = builder.get_object(widget_name).expect("Not found");
        widget.connect_clicked(move |s: &gtk::Button| {
            let name = s.get_widget_name().unwrap();
            let name = name.as_str();
            println!("Sending from {}", name);
            tcmd_clone.send(Cmd::from(name)).expect("Couldn't send cmd");
        });
    }

    // Ram Display
    {
        let ram_display_window: gtk::Window = builder.get_object("RamDisplayWindow").unwrap();
        let switch: gtk::Switch = builder.get_object("RamDisplay").expect("Not Found");
        switch.connect_state_set(move |switch: &gtk::Switch, state: bool| {
            if state {
                ram_display_window.show_all();
            } else {
                ram_display_window.close();
            }
            println!("{}", state);

            Inhibit(false)
        });
        let ram_list: gtk::Window = builder.get_object("RamList").unwrap();
    }

    let registers: gtk::Label = builder.get_object("Registers").unwrap();
    let registers = Mutex::from(registers);

    // Receive GPU page
    {
        let drawing_area = drawing_area.clone();
        let palette = palette.clone();
        let emulator = emulator.clone();
        let img_m = img_m.clone();
        rdata.attach(None, move |data: Vec<_>| {
            println!("Received page");
            for line in data.chunks(16) {
                println!(
                    "{}",
                    line.iter()
                        .map(|h| format!("{:02X}", h))
                        .collect::<String>()
                )
            }
            if let Ok(mut img) = img_m.try_lock() {
                let (w, h) = (img.width, img.height);
                let pixels: &mut [Color] = &mut img.pixels;
                let palette = palette.lock().expect("Couldn't get palette");
                for y in 0..h {
                    for x in 0..w {
                        let i = y * w + x;
                        pixels[i] = palette[data[i] as usize & 0x0F];
                    }
                }
            }
            if let Ok(registers) = registers.try_lock() {
                if let Ok(emulator) = emulator.try_lock() {
                    registers.set_text(&format!("{:#?}", emulator.cpu));
                }
            }
            drawing_area.queue_draw();
            glib::Continue(true)
        });
    }

    // Color palette
    {
        for i in 0..16 {
            let drawing_area = drawing_area.clone();
            let palette = palette.clone();
            let color_button: gtk::ColorButton =
                builder.get_object(&format!("ColorPalette{}", i)).unwrap();
            color_button.connect_color_set(move |s: &gtk::ColorButton| {
                let color = s.get_rgba();
                let color = Color::from((color.red, color.green, color.blue));
                let mut palette = palette.lock().expect("Couldn't acquire palette lock");
                palette[i] = color;
                println!("New color: {:?}", color);
                drawing_area.queue_draw();
            });
            color_button.emit("color-set", &[])?;
        }
    }

    window.set_application(Some(app));
    window.show_all();
    Ok(())
}

fn main() -> Result<(), ProgErr> {
    let app: gtk::Application =
        gtk::Application::new(Some("com.ducklings_corp.emulator"), Default::default())?;
    app.connect_activate(move |app| init(app).expect("Init failed"));
    app.run(&std::env::args().collect::<Vec<_>>());
    Ok(())
}
