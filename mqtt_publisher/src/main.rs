use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::error::Error;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Configuration du client
    let mut options = MqttOptions::new("rust-publisher-1", "192.168.11.119", 1883);
    options
        // 🔧 keepalive 30s : détecte les coupures réseau rapidement
        // sans surcharger avec des PINGs trop fréquents
        .set_keep_alive(Duration::from_secs(30))

        // 🔧 clean_session = FALSE : le broker garde les sub et les
        // messages QoS1 en attente si le capteur se déconnecte
        // → aucun message perdu lors d'une coupure réseau temporaire
        .set_clean_session(false)

        // 🔧 inflight = 20 : aligné avec max_inflight_count du broker
        // Ne jamais dépasser la valeur broker-side
        .set_inflight(20)

        // 🔧 Taille du channel interne : 100 messages en buffer local
        // avant que le send() soit bloquant → absorbe les micro-bursts
        .set_request_channel_capacity(100);


    let (client, mut eventloop) = AsyncClient::new(options, 100);

    let (on , off) = ("ON","OFF");
    let mut state  =  1;
    
    // Tâche qui publie toutes les 2 secondes
    tokio::spawn(async move {
        loop {

            if state == 1 {
                if let Err(e) = client.publish("Commande/S1", QoS::AtMostOnce, false, on.as_bytes()).await {
                    eprintln!("Erreur publish: {}", e);
                    break;
                }
                state ^= 1;
            }else {
                if let Err(e) = client.publish("Commande/S1", QoS::AtMostOnce, false, off.as_bytes()).await {
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

