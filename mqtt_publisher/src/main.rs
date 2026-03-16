use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::error::Error;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configuration du client
    let mut options = MqttOptions::new("rust-publisher-1", "localhost", 1883);
    options.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(options, 10);

    // Tâche qui publie toutes les 2 secondes
    tokio::spawn(async move {
        let mut i = 0;
        loop {
            let payload = format!("Hello from Rust Publisher ! Compteur = {}", i).into_bytes();
            if let Err(e) = client.publish("test/topic", QoS::AtMostOnce, false, payload).await {
                eprintln!("Erreur publish: {}", e);
                break;
            }
            println!("→ Publié : Hello from Rust Publisher ! Compteur = {}", i);
            i += 1;
            time::sleep(Duration::from_secs(2)).await;
        }
    });

    // Boucle qui gère la connexion (obligatoire)
    loop {
        let event = eventloop.poll().await?;
        println!("Event publisher : {:?}", event);
    }
}
