# J'ai identifié 3 bugs dans cette fonction

## Bug 1 — Deadlock : Mutex tenu pendant une opération réseau

state.lock().await garde le verrou pendant toute la durée de disconnect().await. Si la déconnexion réseau prend du temps (timeout, broker lent), aucune autre commande Tauri ne peut accéder à l'état — elles sont toutes bloquées à leur propre state.lock().await. La solution est d'extraire le client, relâcher le verrou, puis faire l'opération réseau.

## Bug 2 — État incohérent : client reste Some après déconnexion

Après disconnect(), s.client contient toujours Some(client_mort). Si l'utilisateur clique ensuite sur ON/OFF, publish_mqtt trouve un Some, croit être connecté, et échoue silencieusement avec une erreur réseau confuse. Il faut remettre s.client = None explicitement.

## Bug 3 — Tâche zombie : la boucle tokio::spawn continue

La boucle de connect_mqtt tourne en arrière-plan en permanence. Quand on déconnecte, disconnect() coupe la connexion réseau, mais le eventloop.poll() reçoit alors une erreur ConnectionAborted et émet mqtt:error au frontend — l'UI affiche une erreur alors qu'on vient juste de se déconnecter volontairement. Il faut un canal d'annulation pour signaler proprement à la boucle de s'arrêter.
