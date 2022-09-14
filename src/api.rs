use super::devmem;
use super::pci;
use actix_web::{get, put, web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ReadDevmemArgs {
    offset: u64,
    length: u64,
}

#[get("/devmem")]
async fn read_devmem(args: web::Query<ReadDevmemArgs>) -> HttpResponse {
    let args = args.into_inner();
    let mut data: Vec<u8> = vec![];
    let start = args.offset;
    let stop = args.offset + args.length;

    for i in start..stop {
        match devmem::read_byte(i) {
            Some(b) => data.push(b),
            None => data.push(0),
        }
    }

    HttpResponse::Ok().body(data)
}

#[derive(Deserialize)]
struct WriteDevmemMeta {
    offset: u64,
    data_type: String,
    data: String,
}

#[put("/devmem")]
async fn write_devmem(meta: web::Json<WriteDevmemMeta>) -> HttpResponse {
    let meta = meta.into_inner();
    let bytes: Option<Vec<u8>> = match meta.data_type.as_str() {
        "byte" => {
            if let Ok(byte) = u8::from_str_radix(&meta.data, 16) {
                Some(vec![byte])
            } else {
                None
            }
        }
        "word" => {
            if let Ok(data) = u16::from_str_radix(&meta.data, 16) {
                Some(data.to_ne_bytes().to_vec())
            } else {
                None
            }
        }
        "dword" => {
            if let Ok(data) = u32::from_str_radix(&meta.data, 16) {
                Some(data.to_ne_bytes().to_vec())
            } else {
                None
            }
        }
        "qword" => {
            if let Ok(data) = u64::from_str_radix(&meta.data, 16) {
                Some(data.to_ne_bytes().to_vec())
            } else {
                None
            }
        }
        "dqword" => {
            if let Ok(data) = u128::from_str_radix(&meta.data, 16) {
                Some(data.to_ne_bytes().to_vec())
            } else {
                None
            }
        }
        _ => None,
    };

    if bytes.is_none() {
        return HttpResponse::NotAcceptable().body(format!(
            "invalid data, type: {}, length: {}",
            meta.data_type,
            meta.data.len()
        ));
    }

    devmem::write(meta.offset, bytes.unwrap());

    HttpResponse::Accepted().body("")
}

#[derive(Serialize, Default)]
struct PciDevice {
    domain: u8,
    bus: u8,
    device: u8,
    function: u8,
    description: String,
}

#[get("/pci/devices")]
async fn get_pci_devices() -> HttpResponse {
    let devices = pci::devices()
        .iter()
        .map(|d| {
            let (device, info) = d;
            let desc = format!(
                "{}: {} {} (rev {})",
                info.class.1.as_ref().unwrap_or(&info.class.0),
                info.vendor.1.as_ref().unwrap_or(&info.vendor.0),
                info.device.1.as_ref().unwrap_or(&info.device.0),
                info.revision,
            );
            PciDevice {
                domain: device.domain,
                bus: device.bus,
                device: device.device,
                function: device.function,
                description: desc,
            }
        })
        .collect::<Vec<PciDevice>>();
    HttpResponse::Ok().json(devices)
}

#[derive(Deserialize)]
struct PciDevQueryArgs {
    domain: u8,
    bus: u8,
    device: u8,
    function: u8,
}

#[get("/pci/device/config")]
async fn get_pci_dev_config(args: web::Query<PciDevQueryArgs>) -> HttpResponse {
    let args = args.into_inner();
    let dev = pci::PciDevice::new(args.domain, args.bus, args.device, args.function);
    match dev.config_data() {
        // Some(data) => {
        //     let hex_array = data
        //         .iter()
        //         .map(|b| format!("{:02x}", b))
        //         .collect::<Vec<String>>();
        //     HttpResponse::Ok().json(hex_array)
        // }
        Some(data) => HttpResponse::Ok().body(data),
        None => HttpResponse::NotFound().body("device not found"),
    }
}
