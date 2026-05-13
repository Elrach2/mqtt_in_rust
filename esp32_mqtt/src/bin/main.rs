#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;

/**/
use esp_hal::gpio::{Level, Output,OutputConfig};
use esp_hal::rng::Rng;
use embassy_net::{
    DhcpConfig, Runner, Stack, StackResources,
    dns::DnsQueryType,
    tcp::TcpSocket,Ipv4Address,
};
use esp_println::{self as _, println};
use esp_radio::wifi::{
    ClientConfig, ModeConfig, ScanConfig, WifiController, WifiDevice, WifiEvent, WifiStaState,
};
// MQTT related imports
use rust_mqtt::{
    client::{client::MqttClient, client_config::ClientConfig as rust_mqtt_client_config},
    packet::v5::reason_codes::ReasonCode,
    utils::rng_generator::CountingRng,
};
use heapless::String;
use log::error;



#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {
        println!("Panic!!!!!");

    }
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.2.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 98768);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    info!("Embassy initialized!");

    let mut led1 = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    let mut led2 = Output::new(peripherals.GPIO4, Level::Low, OutputConfig::default());
    
    // let radio_init = esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let radio_init = &*mk_static!(
        esp_radio::Controller<'static>,
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller")
    );

    let (wifi_controller, interfaces) =
        esp_radio::wifi::new(&radio_init, peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let wifi_interface = interfaces.sta;

    let rng = Rng::new();
    let net_seed = rng.random() as u64 | ((rng.random() as u64) << 32);

    let dhcp_config = DhcpConfig::default();
    let config = embassy_net::Config::dhcpv4(dhcp_config);

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        net_seed,
    );

    // TODO: Spawn some tasks
    let _ = spawner.spawn(connection(wifi_controller));
    let _ = spawner.spawn(net_task(runner));
    let _ = spawner;

    wait_for_connection(stack).await;

    let (on, off) = ("ON", "OFF");
    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;

        let mut rx_buffer = [0; 4096];
        let mut tx_buffer = [0; 4096];

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        //Si tu as un dns -->
//       let address = match stack
//           .dns_query("Elrabtwine", DnsQueryType::A)
//           .await
//           .map(|a| a[0])
//       {
//           Ok(address) => address,
//           Err(e) => {
//               error!("DNS lookup error: {e:?}");
//               continue;
//           }
//       };
        let address = Ipv4Address::new(192, 168, 11, 127); // ← ton IP
        let remote_endpoint = (address, 1884);
        
        info!("connecting...");
        led1.set_high();
        let connection = socket.connect(remote_endpoint).await;
        if let Err(e) = connection {
            error!("connect error: {:?}", e);
            continue;
        }
        info!("connected!");
        led1.set_low();

        let mut config = rust_mqtt_client_config::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            CountingRng(20000),
        );
        config.add_max_subscribe_qos(rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1);
        config.add_client_id("ESP_1");
        // config.clean_session = false; // ← retenir la session entre reconnexions
        config.keep_alive = 15; // secondes — détecte les connexions mortes
        config.max_packet_size = 100;
        let mut recv_buffer = [0; 80];
        let mut write_buffer = [0; 80];

        let mut client =
            MqttClient::<_, 5, _>::new(socket, &mut write_buffer, 80, &mut recv_buffer, 80, config);

        match client.connect_to_broker().await {
            Ok(()) => {}
            Err(mqtt_error) => match mqtt_error {
                ReasonCode::NetworkError => {
                    error!("MQTT Network Error");
                    continue;
                }
                _ => {
                    error!("Other MQTT Error: {:?}", mqtt_error);
                    continue;
                }
            },
        }

        let _ = client.subscribe_to_topic("Commande/S1").await;
        

        loop {
            //Changement de l'etat de la led en fonction de la commande puis envoie du l'etat
            //actuel sur un topic
            let mut topic_str : String<64> = String::new();
            let mut payload_str : String<256> = String::new();
            match client.receive_message().await {

                // Message reçu
                Ok((topic, payload)) => {
                    let _ = topic_str.push_str(topic);
                    let _ = payload_str.push_str(core::str::from_utf8(payload).unwrap_or("<binaire>"));

                    println!("[MQTT] ← [{}] {}", topic_str, payload_str);
                },

                Err(_) => { 
                        error!("MQTT Network Error");
                        break; // ← CRUCIAL : sortir pour recréer le socket
                    } 
            }

            match payload_str.as_str(){
                "ON" =>{
                    if led2.is_set_low(){
                        led2.set_high();
                        match client
                        .send_message(
                            "temperature/1",
                            on.as_bytes(),
                            rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1,
                            true,
                        )
                        .await
                    {
                        Ok(()) => {}
                        Err(mqtt_error) => match mqtt_error {
                            ReasonCode::NetworkError => {
                                error!("MQTT Network Error");
                                continue;
                            }
                            _ => {
                                error!("Other MQTT Error: {:?}", mqtt_error);
                                continue;
                            }
                        },
                    }
                }

                }
                "OFF" =>{
                     if  led2.is_set_high(){
                        led2.set_low();
                        match client
                        .send_message(
                            "temperature/1",
                            off.as_bytes(),
                            rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1,
                            true,
                        )
                        .await
                    {
                        Ok(()) => {}
                        Err(mqtt_error) => match mqtt_error {
                            ReasonCode::NetworkError => {
                                error!("MQTT Network Error");
                                continue;
                            }
                            _ => {
                                error!("Other MQTT Error: {:?}", mqtt_error);
                                continue;
                            }
                        },
                    }
                }
                }
                _ => {}
            }
           
            
            // Switch de la commande pour les tests
            // Timer::after(Duration::from_millis(3000)).await;
        }
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples
}

async fn wait_for_connection(stack: Stack<'_>) {
    println!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    println!("Waiting to get IP address...");
    loop {
        if let Some(config) = stack.config_v4() {
            println!("Got IP: {}", config.address);
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}


#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::sta_state() {
            WifiStaState::Connected => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::Client(
                ClientConfig::default()
                    .with_ssid(SSID.into())
                    .with_password(PASSWORD.into()),
            );
            controller.set_config(&client_config).unwrap();
            println!("Starting wifi");
            controller.start_async().await.unwrap();
            println!("Wifi started!");

            println!("Scan");
            let scan_config = ScanConfig::default().with_max(10);
            let result = controller
                .scan_with_config_async(scan_config)
                .await
                .unwrap();
            for ap in result {
                println!("{:?}", ap);
            }
        }
        println!("About to connect...");

        match controller.connect_async().await {
            Ok(_) => println!("Wifi connected!"),
            Err(e) => {
                println!("Failed to connect to wifi: {:?}", e);
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

// A background task, to process network events - when new packets, they need to processed, embassy-net, wraps smoltcp
#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
