use std::process::Command;
use std::sync::Arc;

fn grab_img(x: u32, y: u32, w: u32, h: u32) -> String {

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("maim -m 10 -g{}x{}+{}+{} -f jpg | base64 -i ", w, h, x, y))
        .output()
        .expect("Failed");

    format!("{},{},{},{}|{}", x,y,w,h, String::from_utf8_lossy(&output.stdout).to_owned())
}

extern crate websocket;

use std::thread;
use websocket::{Server, Message, Sender, Receiver};
use websocket::message::Type;
use websocket::stream::WebSocketStream;
use websocket::header::WebSocketProtocol;

fn main() {
    let server = Server::bind("127.0.0.1:2794").unwrap();

    /// Thread count, set to 1 for no thread spawning
    let thread_count = 4;

    let height: u32 = 1080;
    let width: u32 = 1920;
    let lh: u32 = height / thread_count;
    let lines: u32 = height / lh;

    for connection in server {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            let request = connection.unwrap().read_request().unwrap(); // Get the request
            let headers = request.headers.clone(); // Keep the headers so we can check them

            request.validate().unwrap(); // Validate the request

            println!("Client connected!");

            let mut response = request.accept(); // Form a response

            if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
            	if protocols.contains(&("rust-websocket".to_string())) {
            		// We have a protocol we want to use
            		response.headers.set(WebSocketProtocol(vec!["rust-websocket".to_string()]));
            	}
            }

            let mut client = response.send().unwrap(); // Send the response

            loop {
                let mut threads = vec![];

                if lines == 1 {
                    let msg = Message::text(grab_img(0,0,width,height));
                    client.send_message(&msg).unwrap();
                }

                else { // Do work in threads
                    for y in 0..lines {
                        threads.push(thread::spawn(move || {
                            Message::text(grab_img(0,y*lh,width,lh))
                        }));
                    }

                    for t in threads {
                        let msg = t.join().unwrap();
                        client.send_message(&msg).unwrap();
                    }
                }
            }
        });
    }
}

