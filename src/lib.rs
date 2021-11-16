use sdl2;
use sdl2::video::Window;
use sdl2::pixels::Color;
use ringbuf;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Mod;
use sdl2::render::Texture;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use std::path::Path;
use sdl2::image::LoadSurface;
use sdl2::pixels::PixelFormatEnum;

//const SPRITE_H: u32 = 12;
//const SPRITE_W: u32 = 8;
//const SPRITE_COLS: u32 = 16;
//const SPRITE_ROWS: u32 = 16;
//const SPRITE_SHEET: &'static str = "fonts/BrogueFont1.png";

const SPRITE_H: u32 = 14;
const SPRITE_W: u32 = 9;
const SPRITE_COLS: u32 = 32;
//const SPRITE_ROWS: u32 = 8;
const SPRITE_SHEET: &'static str = "fonts/codepage850t.png";

const SCALE: u32 = 1;

pub struct SdlTTY  {
    // underlying canvas
    canvas: sdl2::render::Canvas<Window>,
    /// wether the window has been closed
    pub is_open: bool,
    /// a buffer containing keycode, modifyer pairs for pressed keys
    /// .poll() must be called to populate this.
    pub input_buffer: ringbuf::Consumer<(Keycode,Mod)>,
    input_buffer_producer: ringbuf::Producer<(Keycode,Mod)>,
    // !!INSAFE TEXTURE!!
    spritesheet: Texture,
    blank: Texture,
    /// the total size of the window
    pub size: (u32,u32),
    /// the main viewport, this stores the cursor data.
    pub main: Port,
    _texture_creator: TextureCreator<WindowContext>
}

//    ________ size
//
//    _      _ margin
//    
//    O#######   | margin |
//    #Text  #            | size
//    ########   | margin |
//
//
#[derive(Debug,Copy,Clone)]
pub struct Port {
    /// a point in the upper left of the port
    pub orgin: (u32,u32),
    /// size the total size the port takes up
    pub size: (u32,u32),
    /// the size margins arround the port
    pub margin: u32,
    /// the cursor position
    pub cursor: (u32,u32),
}

impl Drop for SdlTTY {
    fn drop(&mut self) {
//        unsafe {
//            self.spritesheet.destroy()
//        }
    }
}

impl Port {
    /// gets the dementions inner of the port
    pub fn get_drawable(&self) -> (u32,u32) {
        (self.size.0-self.margin,self.size.1-self.margin)
    }
    /// min/max x in port
    pub fn get_x_range(&self) -> (u32,u32) {
        (self.orgin.0+self.margin,self.orgin.0+self.size.0-self.margin)
    }
    /// min/max y in port
    pub fn get_y_range(&self) -> (u32,u32) {
        (self.orgin.1+self.margin,self.orgin.1+self.size.1-self.margin)
    }
}

impl SdlTTY {
    /// create a new text window from a sdl window
    pub fn new(window: Window) -> SdlTTY {
        let rb = ringbuf::RingBuffer::<(Keycode,Mod)>::new(10);
        let (prod, cons) = rb.split();
        
        let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        
        let texture_creator = canvas.texture_creator();
        
        let mut spritesurf = Surface::from_file(Path::new(SPRITE_SHEET))
            .unwrap();
            
        spritesurf.set_color_key(true,Color::RGB(0,0,0)).unwrap();
        
        //let spritesheet = texture_creator.load_texture(SPRITE_SHEET).unwrap();
        let spritesheet = spritesurf.as_texture(&texture_creator).unwrap();

        let mut blank = Surface::new(1, 1, PixelFormatEnum::RGB888).unwrap();
        blank.fill_rect(None,Color::RGB(255,255,255)).unwrap();
        let blank = blank.as_texture(&texture_creator).unwrap();

        
        return SdlTTY {
            input_buffer_producer: prod,
            input_buffer: cons,
            spritesheet: spritesheet,
            blank,
            _texture_creator: texture_creator,
            canvas,
            main: Port {
                orgin: (0,0),
                size: (0,0),
                margin: 0,
                cursor: (0,0)
            },
            size: (0,0),
            is_open: true,
        };
    }
    /// collect events form a eventpump, this populates input_buffer
    pub fn poll(&mut self, event_pump: &mut sdl2::EventPump) {
        use sdl2::event::Event;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.is_open = false,
                Event::KeyDown {keycode, keymod, ..} => {
                    match keycode {
                        Some(key) => self.input_buffer_producer.push((key,keymod))
                            .unwrap_or_else(|e| println!("[tty] failed to push {:?} to buffer, droping", e)),
                        None => ()
                    }
                },
                _ => {}
            }
        }
        let px_size = self.canvas.output_size().expect("[tty] cant gey window size");
        self.size = (px_size.0/SPRITE_W/SCALE,px_size.1/SPRITE_H/SCALE);
        self.main = self.get_main_port(0);
    }
    /// display the drawn text
    pub fn present(&mut self) {
        self.canvas.present();
    }
    /// clear window and reset cursor
    /// this should be done before drawing
    pub fn clear(&mut self,color: Color) {
        self.canvas.set_draw_color(color);
        self.canvas.clear();
        self.main.cursor = (0,0);
    }
    /// print a single char.
    pub fn putc(&mut self, c :u8,color: Color, pos: (u32,u32),bg: Option<Color>) {
        let x = c as u32 % SPRITE_COLS;
        let y = c as u32 / SPRITE_COLS;
        
        let dest_rect = Rect::new(
            (pos.0 * SPRITE_W*SCALE) as i32, 
            (pos.1 * SPRITE_H*SCALE) as i32, 
            SPRITE_W*SCALE, 
            SPRITE_H*SCALE
        );
        let src_rect = Rect::new(
            (x * SPRITE_W) as i32, 
            (y * SPRITE_H) as i32,
            SPRITE_W,
            SPRITE_H
        );
        
        match bg {
            Some(color) => {
                self.blank.set_color_mod(color.r,color.g,color.b);
                self.canvas.copy(&self.blank, None, dest_rect).expect("[tty] copy failed");
            },
            None => ()
        }
        
        self.spritesheet.set_color_mod(color.r,color.g,color.b);
        self.canvas.copy(&self.spritesheet, src_rect, dest_rect).expect("[tty] copy failed");
        //self.canvas.clear();
    }
    /// compute a port 
    pub fn get_main_port(&self,margin: u32) -> Port {
        Port {
            orgin: (0,0),
            size: self.size,
            margin,
            cursor: (0+margin,0+margin)
        }
    }
    /// draw a box inside the port, this works best with a margin of 1 or higher
    pub fn box_port(&mut self, color: Color,port: &mut Port) {
        for x in 0..port.size.0 {
            self.putc(35,color,(x,port.orgin.1),None);
            self.putc(35,color,(x,port.orgin.1 + port.size.1 - 1),None);
        }
        for y in 0..port.size.1 {
            self.putc(35,color,(port.orgin.0,y),None);
            self.putc(35,color,(port.orgin.0 + port.size.0 - 1,y),None);
        }
    }
    /// print a string using the built in port
    pub fn puts(&mut self, s: &str, color: Color,bg: Option<Color>) {
        let mut main = self.main;
        self.puts_port(s,color,bg,&mut main);
        self.main = main;
    }
    /// print a string using the a specifyed port
    pub fn puts_port(&mut self, s: &str, color: Color, bg: Option<Color>, port :&mut Port) {
        for c in s.as_bytes() {
            match c {
                10 => port.cursor = (port.get_x_range().0,port.cursor.1 + 1),
                c => {
                    //println!("{} {:?}",c,self.cursor);
                    self.putc(*c,color,port.cursor,bg);
                    port.cursor.0 = port.cursor.0 + 1;
                }
            }
        }
    }
    /// draw a string at the top of a box
    pub fn puts_title(&mut self, s: &str, color: Color, bg: Color, port :&mut Port) {
        let mut x = port.get_x_range().0;
        for c in s.as_bytes() {
            self.putc(*c,color,(x,port.orgin.1),Some(bg));
            x += 1;
        }
    }
}