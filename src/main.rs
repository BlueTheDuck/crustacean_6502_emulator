mod error;
use error::ProgErr;

use cairo;
use gio::prelude::*;
use gtk::prelude::*;
use gtk::Builder;

mod emulator;
mod graphic;
mod handler;
use graphic::{Color, Image};
use handler::{Cmd, ThreadedEmulator};

macro_rules! gtk_rs {
    ($builder:expr=>$name:expr) => {
        $builder
            .get_object($name)
            .unwrap_or_else(|| panic!("Object {} could't be found", $name))
    };
}

pub fn init(app: &gtk::Application) -> Result<(), ProgErr> {
    use std::sync::{Arc, Mutex};
    let img_m = Arc::from(Mutex::from(Image::new()));
    let palette: Arc<Mutex<_>> = Arc::from(Mutex::from([Color::default(); 16]));

    let emulator = ThreadedEmulator::new();

    /*
       // Handle emulator CMDs
       let (tcmd, rcmd) = std::sync::mpsc::channel::<Cmd>();
       let (tdata, rdata) = glib::MainContext::channel(glib::source::Priority::default());
       let emulator = Arc::from(Mutex::from(System::new()));
       {
           let emulator = emulator.clone();
           std::thread::spawn(move || loop {
               if let Ok(cmd) = rcmd.recv() {
                   if let Ok(mut emulator) = emulator.try_lock() {
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
                   } else {
                       continue;
                   }
               }
           });
       }



    */
    let builder: Builder = Builder::new_from_string(include_str!("ui.glade"));
    //let window: Option<gtk::ApplicationWindow> = builder.get_object("Window");
    let window: gtk::ApplicationWindow = gtk_rs!(builder=>"Window");
    let drawing_area: gtk::DrawingArea = gtk_rs!(builder=>"Display");
    //let drawing_area = Arc::from(drawing_area);
    {
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
        let tcmd = emulator.tcmd.clone();
        let widget: gtk::Button = gtk_rs!(builder=>widget_name); // builder.get_object(widget_name).expect("Not found");
        widget.connect_clicked(move |s: &gtk::Button| {
            let name = s.get_widget_name().unwrap();
            let name = name.as_str();
            println!("Sending from {}", name);
            tcmd.send(Cmd::from(name)).expect("Couldn't send cmd");
        });
    }

    // Ram Display
    /* {
        let ram_display_window: gtk::Window = gtk_rs!(builder=>"RamDisplayWindow"); // builder.get_object().unwrap();
        let switch: gtk::Switch = gtk_rs!(builder=>"RamDisplay"); // builder.get_object().expect("Not Found");
        switch.connect_state_set(move |switch: &gtk::Switch, state: bool| {
            if state {
                ram_display_window.show_all();
            } else {
                ram_display_window.close();
            }
            println!("{}", state);

            Inhibit(false)
        });
        let ram_list: gtk::Window = gtk_rs!(builder=>"RamList");
    } */

    let registers: gtk::Label = gtk_rs!(builder=>"Registers");

    // Receive GPU page
    {
        let drawing_area = drawing_area.clone();
        let palette = palette.clone();
        let img_m = img_m.clone();
        emulator.rdata.attach(None, move |data: Vec<_>| {
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

            if let Ok(system) = emulator.system.try_lock() {
                registers.set_text(&format!("{:#?}", system.registers));
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
            let color_button = &format!("ColorPalette{}", i);
            let color_button: gtk::ColorButton = gtk_rs!(builder=>color_button);
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
    app.connect_activate(|app| init(app).expect("Init failed"));
    app.run(&std::env::args().collect::<Vec<_>>());
    Ok(())
}
