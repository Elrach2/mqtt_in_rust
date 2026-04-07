use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::error::Error;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configuration du client
    let mut options = MqttOptions::new("rust-publisher-1", "localhost", 1885);
    options.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(options, 10);

    let (on , off) = ("ON","OFF");
    let mut state  =  1;
    
    // Tâche qui publie toutes les 2 secondes
    tokio::spawn(async move {
        loop {

            if state == 1 {
                if let Err(e) = client.publish("test/topic", QoS::AtMostOnce, false, on.as_bytes()).await {
                    eprintln!("Erreur publish: {}", e);
                    break;
                }
                state ^= 1;
            }else {
                if let Err(e) = client.publish("test/topic", QoS::AtMostOnce, false, off.as_bytes()).await {
                    eprintln!("Erreur publish: {}", e);
                    break;
                }
                state ^= 1;
            }

            time::sleep(Duration::from_secs(2)).await;
        }
    });

    // Boucle qui gère la connexion (obligatoire)
    loop {
        let event = eventloop.poll().await?;
        println!("Event publisher : {:?}", event);
    }
}

