#![no_std]
#![no_main]

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> !{
    loop{}
}
esp_bootloader_esp_idf::esp_app_desc!();


use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output,OutputConfig};
// Celle ci nous permettra de faire la temporisation
use esp_hal::timer::timg::TimerGroup;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

/*
* Nous devons préciser le point d'entré du programme car nous n'utilisons pas le std.
* L'attibut #[esp_rtos::main] nous petmet de faire ça
* Il faut ajouter l'expression "async" devant "fn main(spawner: Spawner) -> !" 
*/
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    
    let config      =   esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals  =   esp_hal::init(config); 

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    let led1 = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let led2 = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    
    //
    spawner.spawn(fast_blink(led1));
    spawner.spawn(slow_blink(led2));

    loop{
        Timer::after(Duration::from_secs(1)).await;
    }
}


/*Les taches s'exécutant en parallèle*/

#[embassy_executor::task]
async fn fast_blink(mut led: Output<'static>){
    loop{
        led.toggle();
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task]
async fn slow_blink(mut led: Output<'static>){
    loop{
        Timer::after(Duration::from_secs(1)).await;
        led.toggle();
    }
}
