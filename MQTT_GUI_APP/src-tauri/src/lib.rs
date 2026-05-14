mod mqtt;

use std::sync::Arc;
use tokio::sync::Mutex;
use mqtt::MqttState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    // .manage() = "garde cet objet en mémoire, injecte-le dans les commandes qui en ont besoin"
    .manage(Arc::new(Mutex::new(MqttState::new())))
    
    // Enregistrer toutes les commandes accessibles depuis le frontend
    // Si tu ajoutes une commande dans mqtt.rs, ajoute-la ici aussi
    .invoke_handler(tauri::generate_handler![mqtt::connect_mqtt, mqtt::disconnect_mqtt, mqtt::publish_mqtt,])

    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
