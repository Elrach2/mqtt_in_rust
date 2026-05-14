// On importe les outils nécessaires
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken; // ← ajouter dans Cargo.toml

// ─────────────────────────────────────────────────────────
// ÉTAT PARTAGÉ
// Arc<Mutex<...>> = plusieurs threads peuvent y accéder
// Arc = pointeur partagé, Mutex = verrou (1 seul à la fois)
// ─────────────────────────────────────────────────────────
pub struct MqttState {
    pub client: Option<AsyncClient>, // None = pas connecté
    pub cancel: Option<CancellationToken>, // ← nouveau : pour stopper la boucle
}

impl MqttState {
    pub fn new() -> Self {
        Self { client: None, cancel: None}
    }
}

// ─────────────────────────────────────────────────────────
// COMMANDE 1 : connect_mqtt
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

    // 4. Créer le token d'annulation AVANT de lancer la boucle
    let token = CancellationToken::new();
    let token_clone = token.clone();

    // 5. Lancer une tâche en arrière-plan qui écoute en permanence
    //    tokio::spawn = "lance ça sans bloquer le reste"
    //    async move   = la tâche prend possession des variables
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // ✅ Si le token est annulé → sortir proprement sans émettre d'erreur
                _ = token_clone.cancelled() => {
                println!("Boucle MQTT arrêtée proprement");
                break;
            }

            result = eventloop.poll() => {
                    match result {
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
                                // ✅ N'émettre l'erreur QUE si ce n'est pas une annulation volontaire
                                if !token_clone.is_cancelled() {
                                    let _ = app_clone.emit("mqtt:error", e.to_string());
                                }
                                break;
                        }

                        // Tous les autres événements (PingReq, PingResp...) → ignorer
                        _ => {}
                    }
                }

            }
        }
    });

    // 6. Sauvegarder le client ET le token dans l'état partagé
    //    .lock().await = "je veux accéder au Mutex, j'attends si occupé"
    let mut s = state.lock().await;
    s.client = Some(client);
    s.cancel = Some(token);


    // Ok(()) = succès, pas de valeur de retour
    Ok(())
}

// ─────────────────────────────────────────────────────────
// COMMANDE 2 : publish_mqtt
// ─────────────────────────────────────────────────────────
#[tauri::command]
pub async fn disconnect_mqtt(
    state: tauri::State<'_, Arc<Mutex<MqttState>>>,
    app: AppHandle,
) -> Result<(), String> {
    
    // ✅ Bug 1 corrigé : extraire client et token, puis RELÂCHER le verrou
    let (client, token) = {
        let mut s = state.lock().await;
        let client = s.client.take(); // .take() = retire la valeur → Some→None ✅ Bug 2 corrigé
        let token  = s.cancel.take();
        (client, token)
    }; // ← verrou relâché ici, AVANT disconnect()
    

    match client {
        Some(c) => {
            // ✅ Bug 3 corrigé : annuler la boucle AVANT disconnect()
            if let Some(t) = token {
                t.cancel(); // signal propre à tokio::spawn
            }

            // Maintenant on peut appeler disconnect() sans tenir le verrou
            c.disconnect()
                .await
                .map_err(|e| e.to_string())?;

            let _ = app.emit("mqtt:disconnected", ());
            println!("Déconnecté");
            Ok(())
        }
        None => Err("Pas connecté au broker".to_string()),
    }
}
// ─────────────────────────────────────────────────────────
// COMMANDE 3 : publish_mqtt
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
