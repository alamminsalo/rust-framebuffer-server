use std::process::Command;

fn grab_img(x: u32, y: u32, w: u32, h: u32) -> String {

    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("maim -m 10 -g{}x{}+{}+{} | base64 -i", w, h, x, y))
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

    let height = 768;
    let width: u32 = 768;
    let lh: u32 = 8;
    let lines = height / lh;

    for connection in server {
        // Spawn a new thread for each connection.
        thread::spawn(move || {
            let request = connection.unwrap().read_request().unwrap(); // Get the request
            let headers = request.headers.clone(); // Keep the headers so we can check them

            request.validate().unwrap(); // Validate the request


            let mut response = request.accept(); // Form a response

            if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
            	if protocols.contains(&("rust-websocket".to_string())) {
            		// We have a protocol we want to use
            		response.headers.set(WebSocketProtocol(vec!["rust-websocket".to_string()]));
            	}
            }

            let mut client = response.send().unwrap(); // Send the response

            loop {
                for y in 0..lines {
                    let msg = grab_img(0,y*lh,width,lh);
                    let message: Message = Message::text(msg);
                    client.send_message(&message).unwrap();
                }
            }
        });
    }
}

