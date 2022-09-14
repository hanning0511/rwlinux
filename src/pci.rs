use log::error;
use pciid_parser::Database;
use std::fs;

const SYS_PCI_DEVICE_ROOT: &str = "/sys/bus/pci/devices";

#[derive(Default, Debug, Clone)]
pub struct PciDevBasicInfo {
    pub vendor: (String, Option<String>),
    pub device: (String, Option<String>),
    pub class: (String, Option<String>),
    pub sub_class: (String, Option<String>),
    pub prog_if: (String, Option<String>),
    pub revision: String,
}

#[derive(Default, Debug)]
pub struct PciDevice {
    pub domain: u8,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

impl PciDevice {
    pub fn new(domain: u8, bus: u8, device: u8, function: u8) -> Self {
        Self {
            domain,
            bus,
            device,
            function,
        }
    }

    pub fn from_sysfs_dirname(dirname: &str) -> Option<Self> {
        if dirname.len() != 12 {
            return None;
        }

        let mut pd = Self::default();

        if let Ok(domain) = u8::from_str_radix(&dirname[..4], 16) {
            pd.domain = domain;
        } else {
            error!("fail to parse domain");
            return None;
        }

        if let Ok(bus) = u8::from_str_radix(&dirname[5..7], 16) {
            pd.bus = bus;
        } else {
            error!("fail to parse bus");
            return None;
        }

        if let Ok(device) = u8::from_str_radix(&dirname[8..10], 16) {
            pd.device = device;
        } else {
            error!("fail to parse device");
            return None;
        }

        if let Ok(function) = u8::from_str_radix(&dirname[11..], 16) {
            pd.function = function;
        } else {
            error!("fail to parse function");
            return None;
        }

        Some(pd)
    }

    /// Get PCI device configuration data by reading the sysfs config node
    pub fn config_data(&self) -> Option<Vec<u8>> {
        let config_sys_node = format!(
            "{}/{:04x}:{:02x}:{:02x}.{:01x}/config",
            SYS_PCI_DEVICE_ROOT, self.domain, self.bus, self.device, self.function
        );

        match fs::read(&config_sys_node) {
            Ok(data) => Some(data),
            Err(_) => None,
        }
    }

    /// Read data from sysfs nodes and parse basic PCI device information
    pub fn basic_info(&self) -> Option<PciDevBasicInfo> {
        let mut info = PciDevBasicInfo::default();
        let sys_dev_dir = format!(
            "{}/{:04x}:{:02x}:{:02x}.{:01x}",
            SYS_PCI_DEVICE_ROOT, self.domain, self.bus, self.device, self.function
        );
        let vendor_node = format!("{}/vendor", sys_dev_dir);
        let device_node = format!("{}/device", sys_dev_dir);
        let class_node = format!("{}/class", sys_dev_dir);
        let revision_node = format!("{}/revision", sys_dev_dir);

        // read pci-id database
        let db = Database::read();
        if db.is_err() {
            error!("fail to read pci-ids database");
            return None;
        }
        let db = db.unwrap();

        let vendor_id = fs::read_to_string(vendor_node)
            .unwrap()
            .trim()
            .strip_prefix("0x")
            .unwrap()
            .to_string();
        let device_id = fs::read_to_string(device_node)
            .unwrap()
            .trim()
            .strip_prefix("0x")
            .unwrap()
            .to_string();
        let class_id = fs::read_to_string(class_node)
            .unwrap()
            .trim()
            .strip_prefix("0x")
            .unwrap()
            .to_string();

        if let Some(vendor) = db.vendors.get(&vendor_id) {
            info.vendor = (vendor_id, Some(vendor.name.to_owned()));
            if let Some(device) = vendor.devices.get(&device_id) {
                info.device = (device_id, Some(device.name.to_owned()));
            } else {
                info.device = (device_id, None);
            }
        } else {
            info.vendor = (vendor_id, None);
        }
        if let Some(class) = db.classes.get(&class_id[0..2]) {
            info.class = (class_id[0..2].to_string(), Some(class.name.to_owned()));
            if let Some(sub_class) = class.subclasses.get(&class_id[2..4]) {
                info.sub_class = (class_id[2..4].to_string(), Some(sub_class.name.to_owned()));
                if let Some(prog_if) = sub_class.prog_ifs.get(&class_id[4..6]) {
                    info.prog_if = (class_id[4..6].to_string(), Some(prog_if.to_owned()));
                } else {
                    info.prog_if = (class_id[4..6].to_string(), None);
                }
            } else {
                info.sub_class = (class_id[2..4].to_string(), None);
            }
        } else {
            info.class = (class_id[0..2].to_string(), None);
        }

        info.revision = fs::read_to_string(&revision_node)
            .unwrap()
            .trim()
            .strip_prefix("0x")
            .unwrap()
            .to_string();

        Some(info)
    }
}

pub fn devices() -> Vec<(PciDevice, PciDevBasicInfo)> {
    let mut devices: Vec<(PciDevice, PciDevBasicInfo)> = vec![];

    for entry in fs::read_dir(SYS_PCI_DEVICE_ROOT).unwrap() {
        if entry.is_err() {
            continue;
        }

        let path = entry.unwrap().path();
        let filename = path.file_name();
        if filename.is_none() {
            continue;
        }
        let filename = filename.unwrap();
        let filename = filename.to_str().unwrap();
        match PciDevice::from_sysfs_dirname(filename) {
            Some(device) => {
                let info = device.basic_info();
                if let Some(info) = info {
                    devices.push((device, info));
                }
            }
            _ => {}
        }
    }

    devices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_syf_dirname() {
        let dirname: &str = "0000:02:0f.4";
        let pd = PciDevice::from_sysfs_dirname(dirname).expect("fail to parse data from dirname");
        assert_eq!(pd.domain, 0);
        assert_eq!(pd.bus, 2);
        assert_eq!(pd.device, 15);
        assert_eq!(pd.function, 4);
    }
}
