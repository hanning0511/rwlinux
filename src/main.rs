use rwlinux::pci;

fn main() {
    let dev = pci::PciDevice::from_sysfs_dirname("0000:04:0f.5");
    println!("{:?}", dev);
}
