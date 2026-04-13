# Programmation asynchrone  

La programmation asynchrone nous permet d'executer plusieurs taches en simultané.
Cette abstractions nous permet de gerer le wifi le mqtt et d'autres taches en parallèle.

## .cargo/config.toml et rust-toolchain.toml

Ces fichiers peuvent restet tel qu'ils sont. Pas besion de modification pour le moment.

## Cargo.toml

Nous allons ajouter la fonctionnalité "unstable" a `esp-hal` les crates nécessaires qui sont:
> esp-rtos = { version = "0.2.0", features = ["embassy", "esp32"] }
> embassy-executor = "0.9.1"
> embassy-time = "0.5.0"

## src/main.rs

Il faut :

  ajouter
> use esp_hal::timer::timg::TimerGroup;
> use embassy_executor::Spawner;
> use embassy_time::{Duration, Timer};
  
  Changer l'attribut du point d'entré et la ajouter async devant main  
> #[esp_rtos::main]
> async fn main(spawner: Spawner) -> !

  initialise la configuration CPU, les périphériques, le groupe de timers et l’exécuteur Embassy
> let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
> let peripherals = esp_hal::init(config);
>
> let timg0 = TimerGroup::new(peripherals.TIMG0);
> esp_rtos::start(timg0.timer0);

  Lancer les taches asynchrones avec
> spawner.spawn(fast_blink(led1));
> spawner.spawn(slow_blink(led2));

  Après avoir creer les taches
> #[embassy_executor::task]
> async fn fast_blink(mut led: Output<'static>){
> loop{
> led.toggle();
> Timer::after(Duration::from_millis(100)).await;
> }
> }

---

> **Pour plus d'informations, consultez le lien suivant :**  
> [https://embassy.dev/](https://embassy.dev/)
