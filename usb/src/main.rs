extern crate clap;
extern crate libusb;

mod bridge;
mod config;
mod gdb;
mod utils;
mod usb_bridge;

use clap::{App, Arg};
use config::Config;

use bridge::BridgeKind;
use usb_bridge::UsbBridge;

fn main() {
    let matches = App::new("Wishbone USB Adapter")
        .version("1.0")
        .author("Sean Cross <sean@xobs.io>")
        .about("Bridge Wishbone over USB")
        .arg(
            Arg::with_name("pid")
                .short("p")
                .long("pid")
                .value_name("USB_PID")
                .help("USB PID to match")
                .required_unless("vid")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("vid")
                .short("v")
                .long("vid")
                .value_name("USB_VID")
                .help("USB VID to match")
                .required_unless("pid")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("address")
                .index(1)
                .required(false)
                .help("address to read/write"),
        )
        .arg(
            Arg::with_name("value")
                .index(2)
                .required(false)
                .help("value to write"),
        )
        .arg(
            Arg::with_name("bind-addr")
                .short("a")
                .long("bind-addr")
                .value_name("IP_ADDRESS")
                .help("IP address to bind to")
                .default_value("0.0.0.0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("n")
                .long("port")
                .value_name("PORT_NUMBER")
                .help("Port number to listen on")
                .default_value("1234")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("bridge-kind")
                .short("s")
                .long("server-kind")
                .takes_value(true)
                .possible_values(&["gdb", "wishbone"])
        )
        .get_matches();

    let cfg = Config::parse(matches).unwrap();
    let usb_bridge = UsbBridge::new(&cfg).unwrap();
    usb_bridge.connect().unwrap();

    match cfg.bridge_kind {
        BridgeKind::GDB => {
            loop {
                let mut gdb = gdb::GdbServer::new(&cfg).unwrap();
                loop {
                    if let Err(e) = gdb.process() {
                        println!("Error in GDB server: {:?}", e);
                        break;
                    }
                }
            }
        },
        BridgeKind::Wishbone => {
            ()
        },
        BridgeKind::None => {
            if let Some(addr) = cfg.memory_address {
                if let Some(value) = cfg.memory_value {
                    usb_bridge.poke(addr, value).unwrap();
                } else {
                    let val = usb_bridge.peek(addr).unwrap();
                    println!("Value at {:08x}: {:08x}", addr, val);
                }
            } else {
                panic!("No operation and no address specified!");
            }
        }
    }
}