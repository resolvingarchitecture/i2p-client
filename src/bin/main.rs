extern crate clap;
extern crate dirs;

use clap::{App, Arg, SubCommand};
use i2p_client::I2PClient;
use ra_common::models::{Envelope, Packet, PacketType, NetworkId};

fn main() {
    println!("I2P Client CLI");
    let m = App::new("I2P_Client")
        .about("A SAMv3 I2P client for the local I2P router instance.")
        .version("0.0.18")
        .author("Brian Taylor <brian@resolvingarchitecture.io>")
        .arg(
            Arg::with_name("alias")
                .help("Provides an alias when establishing sessions")
                .short("a")
                .long("alias")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("local")
                .help("use local keys")
                .short("l")
                .long("local")
                .takes_value(true),
        )
        .subcommand(
            SubCommand::with_name("send")
                .help("send message")
                .arg(
                    Arg::with_name("to")
                        .help("alias in address book or b32 address")
                        .short("t")
                        .long("to")
                        .required(true)
                        .takes_value(true),
                )
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
            SubCommand::with_name("receive")
                .help("receive messages")
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
        Some("send") => {
            let msg = String::from(m.value_of("msg").unwrap());
            let to = String::from(m.value_of("to").unwrap());
            send(local, alias, to, msg);
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
        }
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

    println!("Bye...");
}

fn dest(alias: &str) {
    println!("{}\n{}\n", alias, I2PClient::dest(alias));
}

fn aliases() {
    let m = I2PClient::aliases();
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