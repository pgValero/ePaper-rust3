use embedded_svc::wifi::{
    AuthMethod, ClientConfiguration, Configuration, PmfConfiguration, ScanMethod,
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use log::info;

pub fn connect_wifi(
    modem: Modem,
    ssid: &str,
    password: &str,
) -> anyhow::Result<AsyncWifi<EspWifi<'static>>> {
    let sysloop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(
            modem,
            sysloop.clone(),
            Some(EspDefaultNvsPartition::take().unwrap()),
        )
        .unwrap(),
        sysloop,
        timer_service.clone(),
    )?;

    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.parse().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: password.parse().unwrap(),
        channel: None,
        scan_method: ScanMethod::FastScan,
        pmf_cfg: PmfConfiguration::NotCapable,
    });

    wifi.set_configuration(&wifi_configuration)?;

    block_on(wifi.start())?;
    info!("Wifi started");

    block_on(wifi.connect())?;
    info!("Wifi connected");

    block_on(wifi.wait_netif_up())?;
    info!("Wifi netif up");

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    println!("Wifi DHCP info: {:?}", ip_info);

    EspPing::default().ping(
        ip_info.subnet.gateway,
        &esp_idf_svc::ping::Configuration::default(),
    )?;

    Ok(wifi)
}
