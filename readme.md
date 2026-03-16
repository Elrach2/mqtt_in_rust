1. Explication complète du protocole MQTT (lis ça d’abord !)
MQTT = Message Queuing Telemetry Transport
C’est un protocole de messagerie ultra-léger créé en 1999 par IBM pour les connexions à faible bande passante (capteurs, IoT, machines, etc.).
Modèle Publish/Subscribe (pas client-serveur classique)

Il y a un seul serveur appelé broker (Mosquitto dans notre cas).
Les clients se connectent au broker.
Les clients publient (publish) des messages sur des topics (sujets).
Les clients souscrivent (subscribe) à des topics pour recevoir tous les messages publiés dessus.

Exemple concret :
Topic : maison/salon/temperature

Un capteur publie la valeur 22.5 sur ce topic.
Un téléphone et une lampe qui sont abonnés au topic maison/salon/# (le # = wildcard) reçoivent automatiquement le message.

Les topics sont hiérarchiques et peuvent contenir des wildcards :

test/topic
test/# (tout ce qui commence par test/)
+/sensor (n’importe quel premier niveau + /sensor)

Qualité de service (QoS) – très important :

QoS 0 (AtMostOnce) : "fire and forget" → rapide, peut perdre des messages.
QoS 1 (AtLeastOnce) : message arrive au moins une fois (retransmis si besoin).
QoS 2 (ExactlyOnce) : exactement une fois (le plus sûr, mais plus lent).

Autres concepts utiles :

Retained message : le broker garde le dernier message d’un topic (utile pour la température actuelle par exemple).
Last Will : si un client se déconnecte brutalement, le broker publie un message "je suis mort" sur un topic.
Keep Alive : le client envoie un ping toutes les X secondes pour dire "je suis toujours vivant".
Clean Session : si true (défaut), le broker oublie les abonnements quand le client se déconnecte.

Pourquoi c’est génial : très peu de bande passante, fonctionne même sur 3G ou WiFi faible, des millions de clients peuvent se connecter au même broker.

1. Architecture de test que nous allons utiliser

Serveur MQTT → Mosquitto (le plus simple au monde, gratuit, stable).
Clients → écrits en Rust avec la crate rumqttc (la meilleure bibliothèque Rust pure).
Tu vas lancer :
Terminal 1 : le broker Mosquitto
Terminal 2 : le publisher Rust (il envoie des messages toutes les 2 secondes)
Terminal 3 (et autant que tu veux) : le subscriber Rust (il reçoit tout)

1. Installation (à faire une seule fois)
Rust
Va sur <https://rustup.rs> → installe (choisis la version par défaut).
Vérifie : rustc --version et cargo --version
Mosquitto (le serveur)

Linux (Ubuntu/Debian)Bashsudo apt update
sudo apt install mosquitto mosquitto-clients
Windows
Télécharge l’installateur sur <https://mosquitto.org/download/>
Installe, puis lance depuis l’invite de commandes :
"C:\Program Files\mosquitto\mosquitto.exe" -v
macOSBashbrew install mosquitto
brew services start mosquitto   # ou simplement : mosquitto -v

Lance le broker dans un terminal dédié et laisse-le tourner :
Bashmosquitto -v
Tu verras des logs quand des clients se connectent.

1. Comment tester (la partie fun !)

Terminal 1 → lance le broker : mosquitto -v
Terminal 2 → cd mqtt_publisher && cargo run
Terminal 3 → cd mqtt_subscriber && cargo run

Tu vas voir :

Le publisher envoie un message toutes les 2 secondes
Le subscriber reçoit instantanément le message avec le compteur qui augmente

Tu peux ouvrir autant de terminaux subscriber que tu veux (change juste le nom du client rust-subscriber-2, etc.) → tous recevront les messages en même temps. C’est la magie du publish/subscribe !
