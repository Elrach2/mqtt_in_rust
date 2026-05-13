// On importe les outils nécessaires
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

// ─────────────────────────────────────────────────────────
// ÉTAT PARTAGÉ
// Arc<Mutex<...>> = plusieurs threads peuvent y accéder
// Arc = pointeur partagé, Mutex = verrou (1 seul à la fois)
// ─────────────────────────────────────────────────────────
pub struct MqttState {
    pub client: Option<AsyncClient>, // None = pas connecté
}

impl MqttState {
    pub fn new() -> Self {
        Self { client: None }
    }
}

// ─────────────────────────────────────────────────────────
// COMMANDE 1 : connect_mqtt
// Appelée depuis Svelte avec : invoke("connect_mqtt", {...})
// ─────────────────────────────────────────────────────────
#[tauri::command]
pub async fn connect_mqtt(
    host: String,       // "broker.hivemq.com"
    port: u16,          // 1883
    client_id: String,  // "mon-client-123"
    // state = accès à l'état partagé (injecté automatiquement par Tauri)
    state: tauri::State<'_, Arc<Mutex<MqttState>>>,
    // app = handle vers l'application pour pouvoir émettre des événements
    app: AppHandle,
) -> Result<(), String> {
    // 1. Configurer la connexion MQTT
    let mut opts = MqttOptions::new(&client_id, &host, port);
    opts.set_keep_alive(Duration::from_secs(30));

    // 2. Créer le client et la boucle d'événements
    //    client  = pour publier, s'abonner
    //    eventloop = pour recevoir les messages entrants
    let (client, mut eventloop) = AsyncClient::new(opts, 64);

    // 3. Cloner host avant qu'il soit capturé par le spawn
    let host_info = host.clone();
    let app_clone = app.clone();

    // 4. Lancer une tâche en arrière-plan qui écoute en permanence
    //    tokio::spawn = "lance ça sans bloquer le reste"
    //    async move   = la tâche prend possession des variables
    tokio::spawn(async move {
        loop {
            // poll() = "donne-moi le prochain événement MQTT"
            match eventloop.poll().await {

                // Connexion acceptée par le broker
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    println!("Connecté à {}", host_info);
                    // Envoyer l'événement "connecté" au frontend Svelte
                    let _ = app_clone.emit("mqtt:connected", true);
                }

                // Déconnexion propre
                Ok(Event::Incoming(Packet::Disconnect)) => {
                    let _ = app_clone.emit("mqtt:disconnected", ());
                    break; // Sortir de la boucle
                }

                // Erreur réseau ou protocole
                Err(e) => {
                    let _ = app_clone.emit("mqtt:error", e.to_string());
                    break;
                }

                // Tous les autres événements (PingReq, PingResp...) → ignorer
                _ => {}
            }
        }
    });

    // 5. Sauvegarder le client dans l'état partagé
    //    .lock().await = "je veux accéder au Mutex, j'attends si occupé"
    let mut s = state.lock().await;
    s.client = Some(client);

    // Ok(()) = succès, pas de valeur de retour
    Ok(())
}

// ─────────────────────────────────────────────────────────
// COMMANDE 2 : publish_mqtt
// Appelée depuis Svelte avec : invoke("publish_mqtt", {...})
// ─────────────────────────────────────────────────────────
#[tauri::command]
pub async fn publish_mqtt(
    topic:   String, // "Commande/S1"
    payload: String, // "ON" ou "OFF"
    state: tauri::State<'_, Arc<Mutex<MqttState>>>,
) -> Result<(), String> {
    // Accéder au client sauvegardé
    let s = state.lock().await;

    match &s.client {
        Some(client) => {
            // Publier le message
            // QoS::AtLeastOnce = le message sera reçu au moins une fois
            // false = pas de retain (le broker ne garde pas ce message)
            client
                .publish(&topic, QoS::AtLeastOnce, false, payload.as_bytes())
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        // Pas de client = pas connecté
        None => Err("Pas connecté au broker".to_string()),
    }
}
