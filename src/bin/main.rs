extern crate clap;
extern crate dirs;
#[macro_use]
extern crate log;
extern crate simple_logger;

use clap::{App, Arg, SubCommand, AppSettings};
use i2p_client::I2PClient;
use ra_common::models::{Envelope, Packet, PacketType, NetworkId};

fn main() {
    simple_logger::init().unwrap();
    let m = App::new("I2P_Client")
        .about("A SAMv3 I2P client for the local I2P router instance.")
        .version("0.0.22")
        .author("Brian Taylor <brian@resolvingarchitecture.io>")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("alias")
                .help("Provides an alias when establishing sessions")
                .short("a")
                .long("alias")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("local")
                .help("use local keys [true|false]")
                .short("l")
                .long("local")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("ping")
                .help("ping/pong to verify connection to I2P router - not active until 3.2")
                .arg(
                    Arg::with_name("message")
                        .help("message to send as string")
                        .short("m")
                        .long("msg")
                        .required(true)
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("send")
                .help("send message - untested")
                .arg(
                    Arg::with_name("to")
                        .help("b32 address")
                        .short("t")
                        .long("to")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("message")
                        .help("message to send as string - required, max size=31,744 bytes, recommended size is <11KB")
                        .short("m")
                        .long("msg")
                        .min_values(1)
                        .max_values(31_744)
                        .required(true)
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("receive")
                .help("receive messages - untested")
                .arg(
                    Arg::with_name("wait")
                        .help("max time in seconds to wait. default is 0. 255 is max. Not yet working - blocks indefinitely.")
                        .short("w")
                        .long("wait")
                        .takes_value(true)
                        .min_values(0)
                        .max_values(255),
                )
        )
        .subcommand(
            SubCommand::with_name("aliases")
                .help("list aliases")
        )
        .subcommand(
            SubCommand::with_name("dest")
                .help("find a specific destination using nickname (alias/domain)")
                .arg(
                    Arg::with_name("nick")
                        .help("alias for search")
                        .short("n")
                        .long("nick")
                        .required(true)
                        .takes_value(true),
                )
        )
        .subcommand(
            SubCommand::with_name("site")
                .help("retrieve eepsite and save to local specified directory")
                .arg(
                    Arg::with_name("host")
                        .help("host name, e.g. 1m5.i2p")
                        .short("h")
                        .long("host")
                        .required(true)
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("directory")
                        .help("directory to save to")
                        .short("d")
                        .long("dir")
                        .required(true)
                        .takes_value(true),
                )
        )
        .get_matches();

    let mut alias = String::from("Anon"); // default
    if m.value_of("alias").is_some() {
        alias = String::from(m.value_of("alias").unwrap());
    }

    let mut local = true; // default
    if m.value_of("local").is_some() {
        local = m.value_of("local").unwrap().eq("true");
    }

    match m.subcommand_name() {
        Some("ping") => {
            let mut msg = "keep-alive";
            if m.value_of("message").is_some() {
                msg = m.value_of("message").unwrap();
            }
            ping(msg);
        },
        Some("send") => {
            send(local, alias, String::from(m.value_of("to").unwrap()), String::from(m.value_of("msg").unwrap()));
        },
        Some("receive") => {
            let mut wait: u8 = 0; // default
            if m.value_of("wait").is_some() {
                wait = m.value_of("wait").unwrap().parse().unwrap_or(0)
            }
            receive(local, alias, wait);
        },
        Some("aliases") => {
            aliases();
        },
        Some("dest") => {
            if m.value_of("nick").is_some() {
                let nick = m.value_of("nick").unwrap();
                println!("dest nick: {}",nick);
                dest(nick);
            }
        },
        Some("site") => {
            if m.value_of("host").is_some() && m.value_of("directory").is_some() {
                let host = m.value_of("host").unwrap();
                let dir = m.value_of("directory").unwrap();
                site(host, dir);
            }
        },
        None => {
            println!("No subcommand was used")
        },
        _ => println!("Some other subcommand was used"),
    }

    // let alias = String::from("Bob");
    // let mut client = I2PClient::new(true, alias.clone());
    // println!("address ({}): {}", &client.local_dest.len(), &client.local_dest.as_str());
    //
    // let from = client.local_dest.clone();
    //
    // for i in 1..3 {
    //     let env = Envelope::new(0, 0, format!("Message {} from Bob to seed (Alice)", i).into_bytes());
    //     let packet_send = Packet::new(
    //         i,
    //         PacketType::Data as u8,
    //         NetworkId::I2P as u8,
    //         from.clone(),
    //         to.clone(),
    //         Some(env));
    //     println!("Sending msg({})...",i);
    //     client.send(packet_send);
    // }
    //
    // for _i in 1..3 {
    //     match client.receive() {
    //         Ok(packet) => {
    //             if packet.envelope.is_some() {
    //                 println!("msg: {}", String::from_utf8(packet.envelope.unwrap().msg).unwrap().as_str());
    //             }
    //         },
    //         Err(e) => println!("{}", e)
    //     }
    // }

    // client_alice.shutdown();
}

fn ping(msg: &str) {
    let mut client = I2PClient::new(true, String::from("Anon"));

    match client.ping(msg) {
        Some(s) => println!("Pong response: {}",s),
        None => println!("No response")
    }
}

fn dest(alias: &str) {
    println!("{}\n{}\n", alias, I2PClient::dest(alias));
}

fn aliases() {
    let m = I2PClient::aliases();
    println!("Aliases...");
    for k in m.keys() {
        println!("{}\n{}\n", k, m.get(k).unwrap());
    }
}

fn send(use_local: bool, alias: String, to: String, message: String) {
    let mut client = I2PClient::new(use_local, alias);
    let env = Envelope::new(0, 0, message.into_bytes());
    let packet = Packet::new(
        1,
        PacketType::Data as u8,
        NetworkId::I2P as u8,
        client.local_dest.clone(),
        to,
        Some(env));
    println!("Sending msg...");
    client.send(packet);
    println!("Send successful")
}

fn receive(use_local: bool, alias: String, wait: u8) {
    let mut client = I2PClient::new(use_local, alias);
    match client.receive() {
        Ok(packet) => {
            if packet.envelope.is_some() {
                println!("msg received: {}", String::from_utf8(packet.envelope.unwrap().msg).unwrap().as_str());
            }
        },
        Err(e) => println!("{}", e)
    }
}

fn site(host: &str, dir: &str) {
    let mut client = I2PClient::new(true, String::from("Anon"));
    match client.site(host) {

    }
}