// use esp_idf_svc::eventloop::EspSystemEventLoop;
// use esp_idf_svc::hal::modem::Modem;
// use esp_idf_svc::hal::prelude::Peripherals;
// use esp_idf_svc::nvs::EspDefaultNvsPartition;
// use esp_idf_svc::timer::EspTaskTimerService;
// use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
//
// pub struct API<'a> {
//     wifi: AsyncWifi<EspWifi<'a>>,
// }
//
// impl<'a> API<'a> {
//     pub fn new(ssid: &str, password: &str, modem: Modem) -> Self {
//
//         // let peripherals = Peripherals::take().unwrap();
//         let sysloop = EspSystemEventLoop::take().unwrap();
//         let timer_service = EspTaskTimerService::new().unwrap();
//
//         let mut wifi = AsyncWifi::wrap(
//             EspWifi::new(modem, sysloop.clone(), Some(EspDefaultNvsPartition::take().unwrap())).unwrap(),
//             sysloop,
//             timer_service.clone(),
//         ).unwrap();
//
//         Self {
//             wifi,
//         }
//     }
// }