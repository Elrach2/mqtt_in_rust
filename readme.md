Si tu veux un broker 100% en Rust, ça serait bien de commencer avec le broker **rumqttd**.

## Étape 1 : Installer le binaire rumqttd
>
> ```bash
> cargo install --git https://github.com/bytebeamio/rumqtt rumqttd
> ```
>
> (Ça compile depuis le repo officiel. Ça prend 30 secondes à 2 minutes la première fois.)

## Étape 2 : Récupérer le fichier de configuration
>
> ```bash
> curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/bytebeamio/rumqtt/main/rumqttd/rumqttd.toml > rumqttd.toml
> ```

## Étape 3 : Lancement du broker

Dans un terminal dédié (laisse-le ouvert) :
>
> ```bash
> rumqttd --config rumqttd.toml
> ```
>
> Tu verras un beau logo ASCII "RUMQTTD" et des logs qui arrivent quand des clients se connectent.

**NOTE** : Si tu vois des erreurs, c'est sûrement dû au port de connexion car le service mosquitto lancé précédemment utilise le même port que celui spécifié dans le `rumqttd.toml` qui est 1883.
Pour y remédier, il suffit de changer le port dans la configuration `.toml` en mettant le port que tu veux.
Cela implique que tu dois mettre le même port de connexion dans les autres programmes.  
Les clients ne doivent pas avoir le même id car le broker n'accepte qu'un seul client avec le même id.  
