import './style.css'
import './style.css';

// ─────────────────────────────────────────────────────────
// ACCÈS À TAURI
// invoke = appeler une fonction Rust
// listen = écouter un événement venant de Rust
// ─────────────────────────────────────────────────────────
const invoke = (window as any).__TAURI__.core.invoke;
const listen  = (window as any).__TAURI__.event.listen;

// ─────────────────────────────────────────────────────────
// RÉCUPÉRER LES ÉLÉMENTS HTML
// On récupère chaque élément par son id pour pouvoir
// le modifier ou écouter ses événements
// ─────────────────────────────────────────────────────────
const inputHost     = document.getElementById('host')      as HTMLInputElement;
const inputPort     = document.getElementById('port')      as HTMLInputElement;
const inputClientId = document.getElementById('clientId')  as HTMLInputElement;
const btnConnect    = document.getElementById('btn-connect') as HTMLButtonElement;
const btnOn         = document.getElementById('btn-on')    as HTMLButtonElement;
const btnOff        = document.getElementById('btn-off')   as HTMLButtonElement;
const statusEl      = document.getElementById('status')    as HTMLDivElement;
const errorEl       = document.getElementById('error')     as HTMLDivElement;
const lastCmdEl     = document.getElementById('last-cmd')  as HTMLParagraphElement;

// Générer un client ID unique par défaut
inputClientId.value = 'simple-' + Math.random().toString(36).slice(2, 7);

// ─────────────────────────────────────────────────────────
// FONCTIONS UTILITAIRES
// Pour mettre à jour l'interface sans répéter le code
// ─────────────────────────────────────────────────────────
function setStatus(text: string, type: 'ok' | 'ko' | 'neutral' = 'neutral') {
  statusEl.textContent = text;
  statusEl.className   = 'status ' + type;
}

function setError(msg: string) {
  if (msg) {
    errorEl.textContent = msg;
    errorEl.classList.remove('hidden');
  } else {
    errorEl.classList.add('hidden');
  }
}

function setConnected(isConnected: boolean) {
  // Activer/désactiver les boutons selon l'état
  btnConnect.disabled    = isConnected;
  btnOn.disabled         = !isConnected;
  btnOff.disabled        = !isConnected;
  inputHost.disabled     = isConnected;
  inputPort.disabled     = isConnected;
  inputClientId.disabled = isConnected;

  btnConnect.textContent = isConnected ? '✔ Connecté' : '⚡ Connecter';
}

function showLastCmd(cmd: string) {
  lastCmdEl.innerHTML   = `Dernière commande : <strong class="${cmd.toLowerCase()}">${cmd}</strong>`;
  lastCmdEl.classList.remove('hidden');

  // Mettre en évidence le bouton actif
  btnOn.classList.toggle('active',  cmd === 'ON');
  btnOff.classList.toggle('active', cmd === 'OFF');
}

// ─────────────────────────────────────────────────────────
// ÉCOUTER LES ÉVÉNEMENTS RUST
// Rust fait app.emit("mqtt:connected") → on réagit ici
// ─────────────────────────────────────────────────────────
listen('mqtt:connected', () => {
  setConnected(true);
  setStatus('Connecté ✔', 'ok');
  setError('');
});

listen('mqtt:disconnected', () => {
  setConnected(false);
  setStatus('Déconnecté', 'neutral');
});

listen('mqtt:error', (event: any) => {
  // event.payload = valeur passée dans app.emit("mqtt:error", valeur)
  setConnected(false);
  setStatus('Erreur de connexion', 'ko');
  setError(event.payload);
});

// ─────────────────────────────────────────────────────────
// BOUTON CONNECTER
// Au clic → appelle la commande Rust connect_mqtt
// ─────────────────────────────────────────────────────────
btnConnect.addEventListener('click', async () => {
  setStatus('Connexion…', 'neutral');
  setError('');

  try {
    // invoke("nom_commande_rust", { paramètres })
    // Les noms des paramètres doivent correspondre exactement
    // à ceux définis dans #[tauri::command] fn connect_mqtt(host, port, client_id)
    await invoke('connect_mqtt', {
      host:     inputHost.value,
      port:     parseInt(inputPort.value),
      clientId: inputClientId.value,
    });
    // Si pas d'erreur : la connexion est initiée
    // La confirmation arrive via l'événement "mqtt:connected"
  } catch (e: any) {
    setStatus('Erreur', 'ko');
    setError(String(e));
  }
});

// ─────────────────────────────────────────────────────────
// BOUTONS ON / OFF
// Au clic → appelle la commande Rust publish_mqtt
// ─────────────────────────────────────────────────────────
btnOn.addEventListener('click', async () => {
  try {
    await invoke('publish_mqtt', {
      topic:   'Commande/S1',
      payload: 'ON',
    });
    showLastCmd('ON');
  } catch (e: any) {
    setError(String(e));
  }
});

btnOff.addEventListener('click', async () => {
  try {
    await invoke('publish_mqtt', {
      topic:   'Commande/S1',
      payload: 'OFF',
    });
    showLastCmd('OFF');
  } catch (e: any) {
    setError(String(e));
  }
});
