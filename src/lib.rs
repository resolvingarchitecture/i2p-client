extern crate dirs;
extern crate base64;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;

use std::fs::File;

use log::{debug,info,warn};

use std::clone::Clone;
use std::collections::HashMap;
use std::io;
use std::io::{BufReader, Error, ErrorKind, BufRead, Write, Read};
use std::path::{Path};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};

use nom::{IResult};

mod parsers;
use crate::parsers::{datagram_send, datagram_received, gen_reply, pong_received, ping_received, sam_hello, sam_naming_reply, sam_session_status, sam_stream_status};

use ra_common::models::{Packet, Service, Envelope, PacketType, NetworkId};
use ra_common::utils::wait::wait_a_sec;

static DEFAULT_API: &'static str = "127.0.0.1:7656";
// static DEFAULT_UDP_API: &'static str = "127.0.0.1:7655";

static I2P_PID: &'static str = "i2p.pid";
static I2P_STATUS: &'static str = "i2p.status";
static I2P_ADDR_BK: &'static str = "eepsite/docroot/hosts.txt";

static SAM_MIN: &'static str = "3.0";
static SAM_MAX: &'static str = "3.1";

pub enum SigType {
    /// Pubkey 32 bytes; privkey 32 bytes; hash 64 bytes; sig 64 bytes
    EdDsaSha512Ed25519,
    /// Prehash version (double hashing, for offline use such as su3, not for use on the network)
    /// Pubkey 32 bytes; privkey 32 bytes; hash 64 bytes; sig 64 bytes
    EdDsaSha512Ed25519ph,
    /// Blinded version of EdDSA, use for encrypted LS2
    /// Pubkey 32 bytes; privkey 32 bytes; hash 64 bytes; sig 64 bytes
    RedDsaSha512Ed25519
}

impl SigType {
    fn as_string(&self) -> &str {
        match *self {
            SigType::EdDsaSha512Ed25519 => "EDDSA_SHA512_ED25519",
            SigType::EdDsaSha512Ed25519ph => "EDDSA_SHA512_ED25519PH",
            SigType::RedDsaSha512Ed25519 => "REDDSA_SHA512_ED25519",
        }
    }
}

pub enum SessionStyle {
    Datagram,
    Raw,
    Stream,
}

pub struct SamConnection {
    conn: TcpStream,
}

pub struct Session {
    sam: SamConnection,
    local_dest: String,
}

pub struct StreamConnect {
    sam: SamConnection,
    session: Session,
    peer_dest: String,
    peer_port: u16,
    local_port: u16,
}

impl SessionStyle {
    fn string(&self) -> &str {
        match *self {
            SessionStyle::Datagram => "DATAGRAM",
            SessionStyle::Raw => "RAW",
            SessionStyle::Stream => "STREAM",
        }
    }
}

fn verify_received<'a>(vec: &'a [(&str, &str)]) -> Result<HashMap<&'a str, &'a str>, Error> {
    let new_vec = vec.clone();
    let map: HashMap<&str, &str> = new_vec.iter().map(|&(k, v)| (k, v)).collect();
    let res = map.get("RESULT").unwrap_or(&"OK").clone();
    let msg = map.get("MESSAGE").unwrap_or(&"").clone();
    match res {
        "OK" => Ok(map),
        "CANT_REACH_PEER" | "KEY_NOT_FOUND" | "PEER_NOT_FOUND" => {
            Err(Error::new(ErrorKind::NotFound, msg))
        }
        "DUPLICATED_DEST" => Err(Error::new(ErrorKind::AddrInUse, msg)),
        "INVALID_KEY" | "INVALID_ID" => Err(Error::new(ErrorKind::InvalidInput, msg)),
        "TIMEOUT" => Err(Error::new(ErrorKind::TimedOut, msg)),
        "I2P_ERROR" => Err(Error::new(ErrorKind::Other, msg)),
        _ => Err(Error::new(ErrorKind::Other, msg)),
    }
}

fn verify_response<'a>(vec: &'a [(&str, &str)]) -> Result<HashMap<&'a str, &'a str>, Error> {
    let new_vec = vec.clone();
    let map: HashMap<&str, &str> = new_vec.iter().map(|&(k, v)| (k, v)).collect();
    let res = map.get("RESULT").unwrap_or(&"OK").clone();
    let msg = map.get("MESSAGE").unwrap_or(&"").clone();
    match res {
        "OK" => Ok(map),
        "CANT_REACH_PEER" | "KEY_NOT_FOUND" | "PEER_NOT_FOUND" => {
            Err(Error::new(ErrorKind::NotFound, msg))
        }
        "DUPLICATED_DEST" => Err(Error::new(ErrorKind::AddrInUse, msg)),
        "INVALID_KEY" | "INVALID_ID" => Err(Error::new(ErrorKind::InvalidInput, msg)),
        "TIMEOUT" => Err(Error::new(ErrorKind::TimedOut, msg)),
        "I2P_ERROR" => Err(Error::new(ErrorKind::Other, msg)),
        _ => Err(Error::new(ErrorKind::Other, msg)),
    }
}

impl SamConnection {
    fn send<F>(&mut self, msg: String, reply_parser: F) -> Result<HashMap<String, String>, Error>
        where
            F: Fn(&str) -> IResult<&str, Vec<(&str, &str)>>,
    {
        debug!("-> {}", &msg);
        self.conn.write_all(&msg.into_bytes())?;

        let mut reader = BufReader::new(&self.conn);
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        debug!("<- {}", &buffer);

        let response = reply_parser(&buffer);
        let vec = response.unwrap();
        let vec_cmd = vec.0;
        debug!("vec_cmd: {}",vec_cmd);
        let vec_opts = vec.1;
        verify_response(&vec_opts).map(|m| {
            m.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
    }

    fn handshake(&mut self) -> Result<HashMap<String, String>, Error> {
        let hello_msg = format!("HELLO VERSION MIN={} MAX={} \n", SAM_MIN, SAM_MAX);
        self.send(hello_msg, sam_hello)
    }

    fn receive<F>(&mut self, received_parser: F) -> Result<HashMap<String, String>, Error>
        where
            F: Fn(&str) -> IResult<&str, Vec<(&str, &str)>>,
    {
        let mut reader = BufReader::new(&self.conn);
        let mut buffer = String::new();
        reader.read_to_string(&mut buffer)?;
        debug!("<- {}", &buffer);
        let received = received_parser(&buffer);
        let vec_opts = received.unwrap().1;
        verify_received(&vec_opts).map(|m| {
            m.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
    }

    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<SamConnection, Error> {
        let tcp_stream = TcpStream::connect(addr)?;
        let mut socket = SamConnection { conn: tcp_stream };
        socket.handshake()?;
        Ok(socket)
    }

    pub fn naming_lookup(&mut self, name: &str) -> Result<String, Error> {
        let create_naming_lookup_msg = format!("NAMING LOOKUP NAME={} \n", name);
        let ret = self.send(create_naming_lookup_msg, sam_naming_reply)?;
        Ok(ret["VALUE"].clone())
    }

    pub fn gen(&mut self, sig_type: SigType) -> Result<(String,String), Error> {
        let create_gen_msg = format!("DEST GENERATE SIGNATURE_TYPE={} \n", sig_type.as_string());
        let ret = self.send(create_gen_msg, gen_reply)?;
        Ok((ret["PUB"].clone(),ret["PRIV"].clone()))
    }

    pub fn duplicate(&self) -> io::Result<SamConnection> {
        self.conn.try_clone().map(|s| SamConnection { conn: s })
    }

    /// Ping request to peer based on established session
    pub fn ping(&mut self, msg: &str) -> Option<String> {
        match self.send(format!("PING {}", msg), pong_received) {
            Ok(ret) => {
                if ret["PONG"].is_empty() {
                    Some(String::from("Response with no msg"))
                } else {
                    Some(ret["PONG"].clone())
                }
            },
            Err(e) => Some(e.to_string())
        }
    }

    /// Listener waiting for Ping request from peer on established session
    // pub fn pong(&mut self) -> Result<Packet, Error> {
    //     info!("Waiting on remote ping...");
    //     let ret = self.receive(ping_received)?;
    //     let dec_msg = base64::decode(ret["PONG"].clone().into_bytes()).unwrap();
    //     let env = Envelope::new(0, 0, dec_msg);
    //     Ok(Packet::new(
    //         0,
    //         PacketType::Data as u8,
    //         NetworkId::I2P as u8,
    //         ret["FROM"].clone(),
    //         ret["DESTINATION"].clone(),
    //         Some(env)))
    // }

    pub fn send_packet(&mut self, packet: Packet) {
        if packet.envelope.is_some() {
            let env = packet.envelope.unwrap();
            let enc_msg = base64::encode(env.msg);
            let send_env_msg = format!("DATAGRAM SEND FROM={} DESTINATION={} SIZE={} MSG={} \n",
                                       packet.from_addr, packet.to_addr, enc_msg.len(), enc_msg.as_str());
            info!("Sending packet...");
            self.send(send_env_msg, datagram_received).unwrap();
        }
    }

    pub fn recv_packet(&mut self) -> Result<Packet, Error> {
        info!("Waiting on packet...");
        let ret = self.receive(datagram_send)?;
        let dec_msg = base64::decode(ret["MSG"].clone().into_bytes()).unwrap();
        let env = Envelope::new(0, 0, dec_msg);
        Ok(Packet::new(
            0,
            PacketType::Data as u8,
            NetworkId::I2P as u8,
            ret["FROM"].clone(),
            ret["DESTINATION"].clone(),
            Some(env)))
    }
}

impl Session {
    pub fn create<A: ToSocketAddrs>(
        sam_addr: A,
        destination: &str,
        nickname: &str,
        style: SessionStyle,
    ) -> Result<Session, Error> {
        let mut sam = SamConnection::connect(sam_addr).unwrap();
        let create_session_msg = format!("SESSION CREATE STYLE={} ID={} DESTINATION={} \n", style.string(), nickname, destination);
        let ret = sam.send(create_session_msg, sam_session_status)?;
        let local_dest = ret["DESTINATION"].clone();
        // let local_dest = sam.naming_lookup("ME")?;
        Ok(Session { sam, local_dest })
    }

    pub fn sam_api(&self) -> io::Result<SocketAddr> {
        self.sam.conn.peer_addr()
    }

    pub fn naming_lookup(&mut self, name: &str) -> io::Result<String> {
        self.sam.naming_lookup(name)
    }

    pub fn duplicate(&self) -> io::Result<Session> {
        self.sam.duplicate().map(|s| Session {
            sam: s,
            local_dest: self.local_dest.clone(),
        })
    }

    pub fn send_packet(&mut self, packet: Packet) {
        self.sam.send_packet(packet);
    }

    pub fn recv_packet(&mut self) -> Result<Packet,Error> {
        self.sam.recv_packet()
    }

    pub fn ping(&mut self, msg: &str) -> Option<String> {
        self.sam.ping(msg)
    }

    pub fn close(&mut self) {
        self.sam.conn.shutdown(Shutdown::Both).unwrap();
    }
}

impl StreamConnect {
    pub fn new<A: ToSocketAddrs>(
        sam_addr: A,
        destination: &str,
        port: u16,
        nickname: &str,
    ) -> io::Result<StreamConnect> {
        let mut session = Session::create(sam_addr, "TRANSIENT", nickname, SessionStyle::Stream)?;
        let mut sam = SamConnection::connect(session.sam_api()?).unwrap();
        let create_stream_msg = format!("STREAM CONNECT ID={} DESTINATION={} SILENT=false TO_PORT={}\n", nickname, destination, port);
        sam.send(create_stream_msg, sam_stream_status)?;
        let peer_dest = session.naming_lookup(destination)?;
        Ok(StreamConnect { sam, session, peer_dest, peer_port: port, local_port: 0})
    }

    pub fn peer_addr(&self) -> io::Result<(String, u16)> {
        Ok((self.peer_dest.clone(), self.peer_port))
    }

    pub fn local_addr(&self) -> io::Result<(String, u16)> {
        Ok((self.session.local_dest.clone(), self.local_port))
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        self.sam.conn.shutdown(how)
    }

    pub fn duplicate(&self) -> io::Result<StreamConnect> {
        Ok(StreamConnect {
            sam: self.sam.duplicate()?,
            session: self.session.duplicate()?,
            peer_dest: self.peer_dest.clone(),
            peer_port: self.peer_port,
            local_port: self.local_port,
        })
    }
}

impl Read for StreamConnect {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.sam.conn.read(buf)
    }
}

impl Write for StreamConnect {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sam.conn.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.sam.conn.flush()
    }
}

pub enum ClientType {
    Local    = 0,
    Embedded = 1,
}

pub struct I2PClient {
    pub local_dest: String,
    session: Session
}

impl I2PClient {
    pub fn new(use_local: bool, alias: String) -> I2PClient {
        info!("{}", "Initializing I2P Client...");
        // Build paths
        let home = dirs::home_dir().unwrap();
        info!("home directory: {}", home.to_str().unwrap());

        let mut i2p_home = home.clone();
        i2p_home.push(".i2p");
        info!("i2p directory: {}", i2p_home.to_str().unwrap());

        let mut i2p_pid_file = i2p_home.clone();
        i2p_pid_file.push(I2P_PID);
        info!("i2p pid file: {}", i2p_pid_file.to_str().unwrap());

        let mut i2p_status_file = i2p_home.clone();
        i2p_status_file.push(I2P_STATUS);
        info!("i2p status file: {}", i2p_status_file.to_str().unwrap());

        let mut i2p_local_dest_file = i2p_home.clone();
        i2p_local_dest_file.push(alias.clone());
        let i2p_local_dest_path = i2p_local_dest_file.to_str().unwrap();

        let mut dest = String::new();
        let mut local_addr_loaded = false;

        if use_local {
            info!("i2p local dest file: {}", i2p_local_dest_path);

            if Path::new(i2p_local_dest_path).exists() {
                let mut i2p_local_dest_file = File::open(Path::new(i2p_local_dest_path)).unwrap();
                match i2p_local_dest_file.read_to_string(&mut dest) {
                    Ok(len) => {
                        if len > 0 {
                            local_addr_loaded = true;
                            info!("dest from file ({}): {}", len, &dest);
                        } else {
                            info!("{}","dest file empty");
                        }
                    },
                    Err(e) => warn!("{}", e)
                }
            }
        }
        if dest.is_empty() {
            // Establish Session, write to local_dest, and set dest
            match Session::create(DEFAULT_API,
                                  "TRANSIENT",
                                  alias.as_str(),
                                  SessionStyle::Datagram) {
                Ok(session) => {
                    info!("IP: {}, Dest: {}",session.sam_api().unwrap().ip().to_string(), session.local_dest.clone());
                    dest = session.local_dest;
                    if use_local && !local_addr_loaded {
                        info!("Saving dest to file: {}",i2p_local_dest_path);
                        match File::create(i2p_local_dest_path) {
                            Ok(f) => {
                                let mut d_file = f;
                                d_file.write_all(dest.clone().as_bytes()).unwrap();
                            },
                            Err(e) => warn!("{}",e)
                        }
                    }
                },
                Err(err) => {
                    warn!("Error: {}",err.to_string());
                }
            }
        }

        loop {
            info!("{}","Trying to create session...");
            let res = Session::create(DEFAULT_API,
                                      &dest.as_str(),
                                      &alias.as_str(),
                                      SessionStyle::Datagram);
            if res.is_ok() {
                info!("{}", "I2P Client initialized.");
                return I2PClient {
                    local_dest: dest,
                    session: res.unwrap()
                }
            } else {
                warn!("Unable to create Session ({})...waiting a few seconds...", res.err().unwrap());
                wait_a_sec(3)
            }
        }
    }

    pub fn aliases() -> HashMap<String,String> {
        let home = dirs::home_dir().unwrap();
        info!("home directory: {}", home.to_str().unwrap());

        let mut i2p_home = home.clone();
        i2p_home.push(".i2p");
        info!("i2p directory: {}", i2p_home.to_str().unwrap());

        let mut i2p_hosts = i2p_home.clone();
        i2p_hosts.push(I2P_ADDR_BK);
        info!("i2p personal address book: {}", i2p_hosts.to_str().unwrap());

        let mut list: Vec<String> = Vec::new();
        let mut m: HashMap<String,String> = HashMap::new();
        if i2p_hosts.exists() {
            let i2p_hosts_file = File::open(i2p_hosts).unwrap();
            let reader = BufReader::new(i2p_hosts_file);
            let mut first_line_skipped = false;
            for line in reader.lines() {
                if !first_line_skipped {
                    first_line_skipped = true;
                    continue;
                }
                list.push(line.unwrap());
            }
            for s in list {
                let v: Vec<&str> = s.split('=').collect();
                m.insert(String::from(v[0]), String::from(v[1]));
            }
        }
        m
    }

    pub fn dest(alias: &str) -> String {
        match I2PClient::aliases().get(alias) {
            Some(v) => {
                info!("Found alias ({})",alias);
                v.clone()
            },
            None => {
                info!("Alias ({}) not found",alias);
                String::from("None")
            }
        }
    }

    // Send out Packet with optional Envelope
    pub fn send(&mut self, packet: Packet) {
        self.session.send_packet(packet);
    }

    pub fn receive(&mut self) -> Result<Packet, Error> {
        self.session.recv_packet()
    }

    pub fn ping(&mut self, msg: &str) -> Option<String> {
        self.session.ping(msg)
    }

    // pub fn shutdown(&mut self) {
    //     if self.session.is_some() {
    //         self.session.unwrap().close();
    //     }
    // }
}

impl Service for I2PClient {
    fn operate(&mut self, operation: u8, env: Envelope) {
        unimplemented!()
        // let mut packet = Packet::new(1, PacketType::Data as u8, NetworkId::I2P as u8, env.)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
