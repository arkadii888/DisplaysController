#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg(not(windows))]
compile_error!("This agent build is Windows-only. Build on Windows target.");

use enigo::{Enigo, MouseButton, MouseControllable, KeyboardControllable, Key};
use serde::Deserialize;
use tiny_http::{Server, Response, Method, Header};
use std::{
    io::Read,
    process::{Command, Stdio, Child},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

#[derive(Deserialize, Clone, Copy)]
struct Display { left:i32, top:i32, width:i32, height:i32 }

#[derive(Deserialize)]
#[serde(tag="type")]
enum Input {
    #[serde(rename="move")]           Move { x:f64, y:f64, display:Display },
    #[serde(rename="click")]          Click { x:f64, y:f64, display:Display, button:Option<String>, count:Option<u8> },
    #[serde(rename="wheel")]          Wheel { x:f64, y:f64, display:Display, deltaY:f64 },
    #[serde(rename="key")]            Key   { key:String },
    #[serde(rename="return")]         ReturnCursor { display: Option<Display> },
    #[serde(rename="setReturnTarget")] SetReturnTarget { display: Display },
}

fn cors(mut resp: Response<std::io::Cursor<Vec<u8>>>) -> Response<std::io::Cursor<Vec<u8>>> {
    resp.add_header(Header::from_bytes(b"Access-Control-Allow-Origin", b"*").unwrap());
    resp.add_header(Header::from_bytes(b"Access-Control-Allow-Headers", b"content-type").unwrap());
    resp.add_header(Header::from_bytes(b"Access-Control-Allow-Methods", b"POST, OPTIONS, GET").unwrap());
    resp
}

fn map(x:f64, y:f64, d:&Display) -> (i32,i32) {
    let gx = d.left + (x.clamp(0.0,1.0) * d.width as f64) as i32;
    let gy = d.top  + (y.clamp(0.0,1.0) * d.height as f64) as i32;
    (gx, gy)
}

fn move_to_center(enigo: &mut Enigo, d: &Display) {
    let (gx, gy) = map(0.5, 0.5, d);
    enigo.mouse_move_to(gx, gy);
}

#[cfg(windows)]
fn spawn_front_dev(front_dir: &str) -> Option<std::process::Child> {
    use std::process::{Command, Stdio};
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    // запускаем напрямую npm.cmd без "cmd /C", и без окна
    let mut cmd = Command::new("npm.cmd");
    cmd.args(["run", "dev", "--", "--host", "--port", "5173"])
        .current_dir(front_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);

    match cmd.spawn() {
        Ok(child) => Some(child),
        Err(e) => {
            eprintln!("[front] failed to start: {e}");
            None
        }
    }
}

fn wait_front_alive(url: &str, timeout: Duration) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:5173") {
            use std::io::Write;
            let _ = stream.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
    println!("[front] dev server at {url} (expected)");
}

fn main() {
    #[cfg(windows)]
    let _front = spawn_front_dev("../front");

    wait_front_alive("http://127.0.0.1:5173", Duration::from_secs(8));

    let return_target: Arc<Mutex<Option<Display>>> = Arc::new(Mutex::new(None));
    {
        let target = Arc::clone(&return_target);
        thread::spawn(move || {
            use rdev::{listen, Event, EventType, Key};
            let mut ctrl = false;
            let mut meta = false;

            let callback = move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(Key::ControlLeft | Key::ControlRight) => ctrl = true,
                    EventType::KeyRelease(Key::ControlLeft | Key::ControlRight) => ctrl = false,
                    EventType::KeyPress(Key::MetaLeft | Key::MetaRight) => meta = true,
                    EventType::KeyRelease(Key::MetaLeft | Key::MetaRight) => meta = false,
                    EventType::KeyPress(Key::Backspace) => {
                        if ctrl || meta {
                            if let Some(d) = *target.lock().unwrap() {
                                let mut enigo = Enigo::new();
                                move_to_center(&mut enigo, &d);
                            }
                        }
                    }
                    _ => {}
                }
            };
            let _ = listen(callback);
        });
    }

    let server = Server::http("127.0.0.1:27272").unwrap();
    println!("Agent running on http://127.0.0.1:27272");

    for mut req in server.incoming_requests() {
        if req.method() == &Method::Options {
            let _ = req.respond(cors(Response::from_string("")));
            continue;
        }
        if req.method() == &Method::Get && req.url() == "/" {
            let _ = req.respond(cors(Response::from_string("ok")));
            continue;
        }
        if req.method() == &Method::Get && req.url() == "/quit" {
            let _ = req.respond(cors(Response::from_string("bye")));
            break;
        }
        if req.method() != &Method::Post || req.url() != "/input" {
            let _ = req.respond(cors(Response::from_string("invalid")));
            continue;
        }

        let mut body = String::new();
        req.as_reader().read_to_string(&mut body).ok();

        let mut enigo = Enigo::new();
        let mut resp = cors(Response::from_string("ok"));

        match serde_json::from_str::<Input>(&body) {
            Ok(Input::Move { x, y, display }) => {
                let (gx, gy) = map(x, y, &display);
                enigo.mouse_move_to(gx, gy);
            }
            Ok(Input::Click { x, y, display, button, count }) => {
                let (gx, gy) = map(x, y, &display);
                enigo.mouse_move_to(gx, gy);
                let btn = match button.as_deref() {
                    Some("right") => MouseButton::Right,
                    Some("middle") => MouseButton::Middle,
                    _ => MouseButton::Left,
                };
                let n = count.unwrap_or(1).max(1);
                for _ in 0..n { enigo.mouse_click(btn); }
            }
            Ok(Input::Wheel { x, y, display, deltaY }) => {
                let (gx, gy) = map(x, y, &display);
                enigo.mouse_move_to(gx, gy);
                enigo.mouse_scroll_y((-deltaY as i32).clamp(-120, 120));
            }
            Ok(Input::Key { key }) => {
                let k = match key.as_str() {
                    "Enter" => Key::Return,
                    "Escape" => Key::Escape,
                    "Tab" => Key::Tab,
                    _ => Key::Layout(key.chars().next().unwrap_or(' ')),
                };
                enigo.key_click(k);
            }
            Ok(Input::ReturnCursor { display }) => {
                if let Some(d) = display {
                    move_to_center(&mut enigo, &d);
                } else if let Some(d) = *return_target.lock().unwrap() {
                    move_to_center(&mut enigo, &d);
                }
            }
            Ok(Input::SetReturnTarget { display }) => {
                *return_target.lock().unwrap() = Some(display);
            }
            Err(e) => {
                eprintln!("Bad JSON: {e} | body: {body}");
                resp = cors(Response::from_string("bad json").with_status_code(400));
            }
        }

        let _ = req.respond(resp);
    }

    #[cfg(windows)]
    if let Some(mut child) = _front {
        let _ = child.kill();
    }
}
