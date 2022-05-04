use async_recursion::async_recursion;
use dmsg::Messages;
use framebuffer::Framebuffer;
use lazy_static::lazy_static;
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

struct FbDaemon {
    fb: Framebuffer,
    vec: Vec<u8>,
}

impl FbDaemon {
    pub fn new(device: String) -> Result<Self, Box<dyn std::error::Error>> {
        let fb = Framebuffer::new(device)?;
        Ok(Self {
            vec: vec![0; fb.fix_screen_info.smem_len as usize],
            fb,
        })
    }
    pub fn write_pix(&mut self, index: usize, rgb: (u8, u8, u8)) {
        if self.vec.len() > index + 2 {
            self.vec[index] = rgb.0;
            self.vec[index + 1] = rgb.1;
            self.vec[index + 2] = rgb.2;
        }
    }
    pub fn clear_screen(&mut self) {
        self.vec.fill(0);
    }
    pub fn draw_frame(&mut self) {
        self.fb.write_frame(&self.vec);
    }
}

lazy_static! {
    static ref FBDAEMON: Mutex<FbDaemon> = Mutex::new(FbDaemon::new("/dev/fb0".into()).unwrap());
}

#[async_recursion]
async fn handle_msg(msg: Messages) {
    match msg {
        Messages::Write(rgb, index) => {
            let mut fb = FBDAEMON.lock().await;
            fb.write_pix(index, rgb);
        }
        Messages::MVec(vec) => {
            for msg in vec {
                handle_msg(msg).await;
            }
        }
        Messages::Clear => {
            let mut fb = FBDAEMON.lock().await;
            fb.clear_screen();
        }
    }
    let mut fb = FBDAEMON.lock().await;
    fb.draw_frame();
}

async fn handle_stream(mut stream: UnixStream) {
    let mut buf = Vec::with_capacity(1024);
    loop {
        if let Ok(res) = stream.read_to_end(&mut buf).await {
            if res == 0 {
                break;
            }

            match Messages::from_bytes(buf.clone()) {
                Ok(t) => {
                    handle_msg(t).await;
                }
                Err(e) => {
                    println!("{:?}", e);
                }
            };

            buf.clear();
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = UnixListener::bind("/tmp/fbdaemon").unwrap();
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                tokio::spawn(async move {
                    handle_stream(stream).await;
                });
            }
            Err(_) => {}
        }
    }
}
