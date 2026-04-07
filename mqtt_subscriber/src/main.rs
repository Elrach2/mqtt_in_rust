use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut options = MqttOptions::new("rust-subscriber-1", "localhost", 1885);
    options.set_keep_alive(std::time::Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(options, 10);

    // On s'abonne au topic (le # accepte tout sous-test/)
    client.subscribe("test/#", QoS::AtMostOnce).await.unwrap();
    println!("✅ Abonné au topic test/#");

    loop {
        let event = eventloop.poll().await?;
        
        // On affiche seulement les messages publiés
        if let Event::Incoming(Packet::Publish(publish)) = event {
            let payload = String::from_utf8_lossy(&publish.payload);
            match &*payload {
                "ON"    =>  println!("Lampe \"ALLUME\""),
                "OFF"   =>  println!("Lampe \"ETEINT\""),
                _       =>  println!("Lampe \"ALLUME\""),
            }
        }
    }
}
