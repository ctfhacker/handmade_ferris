//! Linux platform for Handmade Ferris


mod dl;

fn main() {

    loop {
        let game_code = dl::get_game_funcs();
        let callme = game_code.callme;
        println!("Callme {:#x}", callme());
        std::thread::sleep_ms(1000);
    }

    panic!();

    let mut window = x11_rs::SimpleWindow::build()
        .x(0)
        .y(0)
        .width(800)
        .height(800)
        .border_width(1)
        .border(0)
        .background(1)
        .finish()
        .expect("Failed to create X11 simple window");

    window.create_image();

    for col in 0..800 {
        for row in 0..800 {
            let index = col * 800 + row;
            let color = (col % 256) << 8 | (row % 256);
            window.framebuffer[index] = u32::try_from(color).unwrap();
        }
    }

    window.put_image();

    let mut offset = 0;

    

    // Main event loop
    loop {
        let event = window.next_event();
        println!("Event: {:?}", event);

        match event {
            x11_rs::Event::Expose => window.put_image(),
            x11_rs::Event::KeyPress => {
                println!("Key pressed");
            }
            _ => {
                for col in 0..800 {
                    for row in 0..800 {
                        let index = col * 800 + row;
                        let color = ((col + offset) % 256) << 8 | ((row + offset) % 256);
                        window.framebuffer[index] = u32::try_from(color).unwrap();
                    }
                }
                window.put_image();
                offset += 1;
            }
        }
    }
}
