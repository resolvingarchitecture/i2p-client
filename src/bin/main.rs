extern crate clap;
extern crate dirs;
extern crate log;
extern crate simple_logger;

use clap::{crate_version, App, Arg, AppSettings};
use i2p_client::{I2PClient, SigType};

fn main() {
    simple_logger::init().unwrap();
    let m = App::new("i2p")
        .about("A SAM I2P client for the local I2P router instance. Not compliant with any version yet.")
        .version(crate_version!())
        .author("Brian Taylor <brian@resolvingarchitecture.io>")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::ColorAlways)
        .arg(
            Arg::with_name("alias")
                .help("Provides an alias when establishing sessions. If not provided, will use 'Anon' by default.")
                .short("a")
                .long("alias")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("min_version")
                .help("Minimum SAM version")
                .long("min")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("max_version")
                .help("Maximum SAM version")
                .long("max")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("max_connection_attempts")
                .help("Maximum attempts to make a connection before failure is accepted. Each failed attempt results in waiting 3 seconds prior to making another attempt.")
                .short("c")
                .long("max_connection_attempts")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("local")
                .help("use local keys [true|false]; true by default; when true, it will use an internally saved keyset with provided alias if provided or 'Anon' if not - when set to false, it uses whatever the I2P router provides")
                .short("l")
                .long("local")
                .takes_value(true),
        )
        .subcommand(
            App::new("aliases")
                .about("list aliases")
        )
        .subcommand(
            App::new("dest")
                .about("find a specific destination using nickname (alias/domain)")
                .arg(
                    Arg::with_name("dest_alias")
                        .help("alias for destination search")
                        .short("da")
                        .long("dest_alias")
                        .required(true)
                        .takes_value(true),
                )
        )
        .subcommand(
            App::new("gen")
                .about("generate pub/priv keys; default sig_type is: DSA_SHA1; uses sig_type if provided - values accepted:\n\tDSA_SHA1\n\tEDDSA_SHA512_ED25519\n\tEDDSA_SHA512_ED25519PH\n\tREDDSA_SHA512_ED25519")
                .arg(
                    Arg::with_name("sig_type")
                        .help("Signature Type")
                        .short("s")
                        .long("sig_type")
                        .takes_value(true),
                )
        )
        .subcommand(
            App::new("send")
                .about("send message - not verified to be working; max message size=31,744 bytes, recommended size is <11KB")
                .args(&[
                    Arg::with_name("to")
                        .help("b32 address")
                        .long("to")
                        .required(true)
                        .takes_value(true),
                    Arg::with_name("message")
                        .help("message to send as string - required, max size=31.5KB, recommended size is <11KB")
                        .long("message")
                        .required(true)
                        .takes_value(true),
                    Arg::with_name("max_attempts")
                        .help("maximum number of sends until an ack is received")
                        .long("max_attempts")
                        .takes_value(true),
                ])
        )
        .subcommand(
            App::new("receive")
                .about("receive messages - not receiving messages yet")
        )
        // .subcommand(
        //     SubCommand::with_name("ping")
        //         .help("ping/pong to verify connection to I2P router - not active until SAMv3.2 supported")
        //         .arg(
        //             Arg::with_name("message")
        //                 .help("message to send as string")
        //                 .short("msg")
        //                 .long("message")
        //                 .required(true)
        //                 .takes_value(true),
        //         )
        // )
        // .subcommand(
        //     SubCommand::with_name("site")
        //         .help("retrieve eepsite and save to local specified directory")
        //         .arg(
        //             Arg::with_name("host")
        //                 .help("host name, e.g. 1m5.i2p")
        //                 .short("h")
        //                 .long("host")
        //                 .required(true)
        //                 .takes_value(true),
        //         )
        //         .arg(
        //             Arg::with_name("directory")
        //                 .help("directory to save to")
        //                 .short("d")
        //                 .long("dir")
        //                 .required(true)
        //                 .takes_value(true),
        //         )
        // )
        .get_matches();

    let local = true; // default
    // if m.value_of("local").is_some() {
    //     local = m.value_of("local").unwrap().eq("true");
    // }
    let mut alias = String::from("Anon"); // default
    if m.value_of("alias").is_some() {
        alias = String::from(m.value_of("alias").unwrap());
    }
    let mut min_version = "3.0"; // default
    if m.value_of("min_version").is_some() {
        min_version = m.value_of("min_version").unwrap();
    }
    let mut max_version = "3.1"; // default
    if m.value_of("max_version").is_some() {
        max_version = m.value_of("max_version").unwrap();
    }
    let mut max_connection_attempts: u8 = 3; // default
    if m.value_of("max_connection_attempts").is_some() {
        max_connection_attempts = m.value_of("max_connection_attempts").unwrap().parse().unwrap();
    }

    match m.subcommand_name() {
        Some("aliases") => {
            aliases();
        },
        Some("gen") => {
            let am = m.subcommand().1.unwrap();
            let mut sig_type = "DSA_SHA1";
            if am.value_of("sig_type").is_some() {
                sig_type = am.value_of("sig_type").unwrap();
            }
            gen(
                sig_type,
                local,
                alias,
                min_version,
                max_version,
                max_connection_attempts);
        },
        Some("dest") => {
            dest(m.subcommand().1.unwrap().value_of("dest_alias").unwrap());
        },
        Some("send") => {
            let am = m.subcommand().1.unwrap();
            let mut max_attempts :u8 = 1;
            if am.value_of("max_attempts").is_some() {
                max_attempts = am.value_of("max_attempts").unwrap().parse().unwrap();
            }
            send(
                String::from(am.value_of("to").unwrap()),
                String::from(am.value_of("message").unwrap()),
                max_attempts,
                local,
                alias,
                min_version,
                max_version,
                max_connection_attempts);
        },
        Some("receive") => {
            receive(
                local,
                    alias,
                    min_version,
                    max_version,
                    max_connection_attempts);
        },
        // Some("ping") => {
        //     let mut msg = "keep-alive";
        //     if m.value_of("message").is_some() {
        //         msg = m.value_of("message").unwrap();
        //     }
        //     ping(
        //         msg,
        //          local,
        //          alias,
        //          min_version,
        //          max_version,
        //          max_connection_attempts);
        // },
        // Some("site") => {
        //     if m.value_of("host").is_some() && m.value_of("directory").is_some() {
        //         let host = m.value_of("host").unwrap();
        //         let dir = m.value_of("directory").unwrap();
        //         site(host,
        //              dir,
        //              local,
        //              alias,
        //              min_version,
        //              max_version,
        //              max_connection_attempts);
        //     }
        // },
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

fn aliases() {
    let m = I2PClient::aliases();
    println!("Aliases...");
    for k in m.keys() {
        println!("{}\n{}\n", k, m.get(k).unwrap());
    }
}

fn dest(alias: &str) {
    println!("{}\n{}\n", alias, I2PClient::dest(alias));
}

fn gen(sig_type: &str, use_local: bool, alias: String, min_version: &str, max_version: &str, max_connection_attempts: u8) {
    match SigType::from_str(sig_type) {
        Ok(sig_type) => {
            match I2PClient::new(use_local, alias, min_version, max_version, max_connection_attempts)
            {
                Ok(mut client) => {
                    match client.gen(sig_type) {
                        Ok(t) => {
                            println!("public key:\n{}\nprivate key:\n{}", t.0, t.1)
                        },
                        Err(e) => println!("{}", e)
                    }
                },
                Err(e) => println!("{}", e)
            }
        },
        Err(e) => println!("{}", e)
    }
}

fn send(to: String, message: String, max_attempts: u8, use_local: bool, alias: String, min_version: &str, max_version: &str, max_connection_attempts: u8) {
    match I2PClient::new(use_local, alias, min_version, max_version, max_connection_attempts) {
        Ok(mut client) => {
            for i in 0..max_attempts {
                println!("Sending msg...");
                client.send(to.clone(), message.clone().into_bytes());
                println!("Send successful")
            }
        },
        Err(e) => println!("{}", e)
    }
}

fn receive(use_local: bool, alias: String, min_version: &str, max_version: &str, max_connection_attempts: u8) {
    match I2PClient::new(use_local, alias, min_version, max_version, max_connection_attempts) {
        Ok(mut client) => {
            match client.receive() {
                Ok(tup) => {
                    match String::from_utf8(tup.1) {
                        Ok(msg) => println!("msg received: {}\nfrom: {}", msg, tup.0),
                        Err(e) => println!("{}", e)
                    }
                },
                Err(e) => println!("{}", e)
            }
        },
        Err(e) => println!("{}", e)
    }
}

// fn ping(msg: &str, use_local: bool, alias: String, min_version: &str, max_version: &str, max_connection_attempts: u8) {
//     let mut client = I2PClient::new(use_local, alias, min_version, max_version, max_connection_attempts);
//     match client.ping(msg) {
//         Some(s) => println!("Pong response: {}",s),
//         None => println!("No response")
//     }
// }

// fn site(host: &str, dir: &str, use_local: bool, alias: String, min_version: &str, max_version: &str, max_connection_attempts: u8) {
    // let mut client = I2PClient::new(use_local, alias, min_version, max_version, max_connection_attempts);
    // match client.site(host) {
    //
    // }
// }