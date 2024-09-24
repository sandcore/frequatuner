use esp_idf_hal::{delay::FreeRtos, modem::Modem};
use esp_idf_svc::{
    wifi::EspWifi,
    nvs::EspDefaultNvsPartition,
    eventloop::EspSystemEventLoop,
};
use embedded_svc::wifi::{ClientConfiguration, Configuration};

/*
Wifi was just used for some UDP audio streaming in current project as a test of audio input peripherals.
It assumes working wifi station and valid username and password.
 */
pub fn boot_get_driver<'a>(modem: &'a mut Modem, ssid: &str, password: &str) -> EspWifi<'a> {
    let mut heapless_ssid: heapless::String<32> = heapless::String::new();
    let mut heapless_passwd: heapless::String<64> = heapless::String::new();
    heapless_ssid.push_str(&ssid);
    heapless_passwd.push_str(&password);

    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let mut wifi_driver = EspWifi::new(modem, sys_loop, Some(nvs)).unwrap();

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration{
        ssid: heapless_ssid,
        password: heapless_passwd,
        ..Default::default()
    })).unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap(){
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        FreeRtos::delay_ms(3000); // give OS a chance to do some threading
    }

    println!("Should be connected");
    println!("IP info: {:?}", wifi_driver.sta_netif().get_ip_info().unwrap());
    let ip = wifi_driver.sta_netif().get_ip_info().unwrap().ip;
    let ip_0 = ip.octets()[0];
    let ip_1 = ip.octets()[1];
    let ip_2 = ip.octets()[2];
    let ip_3 = ip.octets()[3];
    let ip_string = ip_0.to_string()+"."+&ip_1.to_string()+"."+&ip_2.to_string()+"."+&ip_3.to_string()+":5005";
    println!("{}", ip_string);

    wifi_driver
}