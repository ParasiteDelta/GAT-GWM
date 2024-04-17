use serde_json::Value;
use tungstenite::{connect, Message};
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (mut socket, response) =
    connect(Url::parse("ws://localhost:6123").unwrap()).expect("Can't connect");

    println!("Connected to GWM\nResCode - {}", response.status());
    println!("Response Headers:\n");
    for (ref header, _value) in response.headers() {
        println!("* {}", header);
    }

    socket.send(Message::Text("subscribe -e window_managed".into())).unwrap();
    loop {
        let msg = socket.read().expect("ERR: Could not read message!\n");
        println!("\n{}\n", msg);
        let json_msg: Value = match serde_json::from_str(msg.to_text().unwrap()) {
            Ok(v) => v,
            Err(_) => continue
        };

        match json_msg["data"]["managedWindow"]["sizePercentage"].as_f64() {
            Some(f) => {
                if f <= 0.5 {
                    socket.send(
                        Message::Text(String::from("command \"tiling direction toggle\""))
                    ).unwrap();
                }
            },
            None => continue
        }
    }

    Ok(())
}