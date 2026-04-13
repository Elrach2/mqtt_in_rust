//La premiere ligne est indispensable pour le bon fonctionnement du programme
#![no_std]
//Cette attribut nous dit que nous n'utiliserons pas le point d'entré standard
#![no_main]

//Le panic_handler s'execute lorsqu'une erreur survient
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> !{
    loop{}
}
// Pour la generation du descripteur d'application
esp_bootloader_esp_idf::esp_app_desc!();


//La ligne suivante nous donne l'attribut qui nous permettra de spécifié le point d'entre de notre
//programme
use esp_hal::main;
use esp_hal::clock::CpuClock;
// Cette inclusion nous permet de configurer les gpio ou pin
use esp_hal::gpio::{Level, Output,OutputConfig};
// Celle ci nous permettra de faire la temporisation
use esp_hal::time::{Duration, Instant};


/*Si tu veux tu peux utiliser un crate pour gerer les paniques mais ici nous allons ecrir notre
* fonction qui gere  les paniques. En ce moment il faut ajouter le crate dans le Cargo.toml
* use panic_halt as _; 
*/

/*
* Nous devons préciser le point d'entré du programme car nous n'utilisons pas le std.
* L'attibut #[main] nous petmet de faire ça
* Il est obligatoire que cette fonction ne retourne rien   " () -> ! "
* Donc nous devons mettre une boucle infinie. Nous avons le choix entre le loop{} ou l'expression suivante ou son équivalent
*   while true {
*   }
*   panic!("..."); 
*/
#[main]
fn main() -> ! {
    
    let config      =   esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripheral  =   esp_hal::init(config); 

    let mut led     =   Output::new(peripheral.GPIO2, Level::High, OutputConfig::default());
    
    loop{
        led.toggle();
        blocking_time(Duration::from_millis(1000));
    }
}

// fonction qui sert a la temporisation

fn blocking_time(duration : Duration ) {
    let delay_start     =   Instant::now();
    while delay_start.elapsed() < duration {
        //attend
    }
}

/* Pour plus d'info consulter le lien suivant:  https://esp32.implrust.com/std-to-no-std/index.html */
