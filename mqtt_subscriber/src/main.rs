use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut options = MqttOptions::new("rust-subscriber-1", "192.168.11.140", 1883);
    options   
        .set_keep_alive(std::time::Duration::from_secs(30))
        // 🔧 clean_session = FALSE pour un dashboard permanent :
        // reçoit les messages manqués pendant une déco
        .set_clean_session(false)
        .set_inflight(20)
        .set_request_channel_capacity(200);  // Plus grand : le dashboard reçoit beaucoupoptions

    let (client, mut eventloop) = AsyncClient::new(options, 200);

    // On s'abonne au topic (le # accepte tout sous-test/)
    client.subscribe("Commande/S1", QoS::AtMostOnce).await.unwrap();
    println!("✅ Abonné au topic temperature/1");

    loop {
        let event = eventloop.poll().await?;
        
        // On affiche seulement les messages publiés
        if let Event::Incoming(Packet::Publish(publish)) = event {
            let payload = String::from_utf8_lossy(&publish.payload);
            println!("{}", payload);
            match &*payload {
                "ON"    =>  println!("Lampe \"ALLUME\""),
                "OFF"   =>  println!("Lampe \"ETEINT\""),
                _       =>  println!("Lampe \"ALLUME\""),
            }
        }
    }
}
