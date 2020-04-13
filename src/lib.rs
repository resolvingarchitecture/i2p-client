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
use std::convert::{TryFrom};
use std::{io, thread};
use std::io::{BufReader, Error, ErrorKind, BufRead, Write, Read, Cursor};
use std::path::{Path};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};

use nom::{IResult};

mod parsers;
use crate::parsers::{datagram_send, datagram_received, gen_reply, pong_received, ping_received, sam_hello, sam_naming_reply, sam_session_status, sam_stream_status};
use std::time::Duration;

static DEFAULT_API: &'static str = "127.0.0.1:7656";
// static DEFAULT_UDP_API: &'static str = "127.0.0.1:7655";

static I2P_PID: &'static str = "i2p.pid";
static I2P_STATUS: &'static str = "i2p.status";
static I2P_ADDR_BK: &'static str = "eepsite/docroot/hosts.txt";

#[derive(Debug, Copy, Clone)]
pub enum SigType {
    /// Pubkey 32 bytes; privkey 32 bytes; hash 64 bytes; sig 64 bytes
    EdDsaSha512Ed25519,
    /// Prehash version (double hashing, for offline use such as su3, not for use on the network)
    /// Pubkey 32 bytes; privkey 32 bytes; hash 64 bytes; sig 64 bytes
    EdDsaSha512Ed25519ph,
    /// Blinded version of EdDSA, use for encrypted LS2
    /// Pubkey 32 bytes; privkey 32 bytes; hash 64 bytes; sig 64 bytes
    RedDsaSha512Ed25519,
    DsaSha1
}

impl SigType {
    pub fn as_string(&self) -> &'static str {
        match *self {
            SigType::EdDsaSha512Ed25519 => "EDDSA_SHA512_ED25519",
            SigType::EdDsaSha512Ed25519ph => "EDDSA_SHA512_ED25519PH",
            SigType::RedDsaSha512Ed25519 => "REDDSA_SHA512_ED25519",
            SigType::DsaSha1 => "DSA_SHA1"
        }
    }
    pub fn from_str(sig_type: &str) -> Result<Self, Error> {
        match sig_type {
            "EDDSA_SHA512_ED25519" => Ok(SigType::EdDsaSha512Ed25519),
            "EDDSA_SHA512_ED25519PH" => Ok(SigType::EdDsaSha512Ed25519ph),
            "REDDSA_SHA512_ED25519" => Ok(SigType::RedDsaSha512Ed25519),
            "DSA_SHA1" => Ok(SigType::DsaSha1),
            _ => Result::Err(Error::new(ErrorKind::InvalidData, format!("SigType provided not supported: {}", sig_type)))
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

pub struct SamConnection {
    last_send_id: u8,
    last_receive_id: u8,
    conn: TcpStream,
    min_version: String,
    max_version: String,
    current_version: String
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
        let vec_opts = vec.1;
        verify_response(&vec_opts).map(|m| {
            m.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
    }

    fn send_async(&mut self, msg: String) {
        debug!("-> {}", &msg);
        match self.conn.write_all(&msg.into_bytes()) {
            Ok(m) => debug!("{}", "msg written to conn"),
            Err(e) => warn!("{}", e)
        }
    }

    fn handshake(&mut self) -> Result<HashMap<String, String>, Error> {
        let hello_msg = format!("HELLO VERSION MIN={} MAX={} \n", self.min_version, self.max_version);
        self.send(hello_msg, sam_hello)
    }

    fn receive<F>(&mut self, received_parser: F) -> Result<HashMap<String, String>, Error>
        where
            F: Fn(&str) -> IResult<&str, Vec<(&str, &str)>>,
    {
        let mut reader = BufReader::new( &self.conn);

        let mut header = String::new();
        let mut ack = String::new();
        let mut body = String::new();
        reader.read_line(&mut header)?;
        reader.read_line(&mut ack)?;
        reader.read_line(&mut body)?;
        debug!("<- (header) {}", &header);
        debug!("<- (ack) {}", &ack);
        let received = received_parser(&header);
        let vec = received.unwrap();
        let mut vec_opts = vec.1;
        vec_opts.push(("ACK", ack.as_str()));
        vec_opts.push(("MSG", body.as_str()));
        verify_received(&vec_opts).map(|m| {
            m.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
    }

    pub fn connect<A: ToSocketAddrs>(addr: A, min_version: &str, max_version: &str) -> Result<SamConnection, Error> {
        let tcp_stream = TcpStream::connect(addr)?;
        let mut conn = SamConnection {
            last_send_id: 0,
            last_receive_id: 0,
            conn: tcp_stream ,
            min_version: String::from(min_version),
            max_version: String::from(max_version),
            current_version: String::from("3.0")
        };
        match conn.handshake() {
            Ok(m) => {
                if !m["VERSION"].is_empty() {
                    conn.current_version = m["VERSION"].clone();
                }
                if m["RESULT"].eq("NOVERSION") {
                    return Err(Error::new(ErrorKind::InvalidInput, "No version"));
                }
                if m["RESULT"].eq("I2P_ERROR") {
                    return Err(Error::new(ErrorKind::ConnectionRefused, m["MESSAGE"].clone()));
                }
                return Ok(conn)
            },
            Err(e) => return Err(e)
        }
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
        self.conn.try_clone().map(|s| SamConnection {
            last_send_id: self.last_send_id,
            last_receive_id: self.last_receive_id,
            conn: s,
            min_version: self.min_version.clone(),
            max_version: self.max_version.clone(),
            current_version: self.current_version.clone() })
    }

    /// Ping request to peer based on established session
    // pub fn ping(&mut self, msg: &str) -> Option<String> {
    //     match self.send(format!("PING {}", msg), pong_received) {
    //         Ok(ret) => {
    //             if ret["PONG"].is_empty() {
    //                 Some(String::from("Response with no msg"))
    //             } else {
    //                 Some(ret["PONG"].clone())
    //             }
    //         },
    //         Err(e) => Some(e.to_string())
    //     }
    // }

    // pub fn site(&mut self, host_dest: &str) -> Result<Vec<u8>, Error> {
    //     let local_dest = "";
    //     let send_env_msg = format!("DATAGRAM SEND FROM={} DESTINATION={} SIZE={} MSG={} \n",
    //                                local_dest, host_dest, 0, "");
    //     info!("Sending site request...");
    //     self.send(send_env_msg, datagram_received).unwrap()
    // }

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

    pub fn send_msg(&mut self, to: String, msg: Vec<u8>, ack: u8) {
        let enc_msg = base64::encode(msg);
        let send_env_msg = format!("DATAGRAM SEND DESTINATION={} SIZE={} \n{}\n{}", to, enc_msg.len(), ack, enc_msg.as_str());
        if send_env_msg.len() > 31_500 {
            warn!("Message length is greater than 31.5KB; recommended to stay below this and ideally less than 11KB.")
        } else if send_env_msg.len() > 61_500 {
            warn!("Unable to send messages greater than 61.5KB (tunnel limit). Rejecting.");
            return;
        }
        info!("Sending packet (size={})...", send_env_msg.len());
        self.send_async(send_env_msg);
        info!("Msg sent.");
    }

    pub fn recv_msg(&mut self) -> Result<(String,Vec<u8>), Error> {
        info!("Waiting on msg...");
        let res = self.receive(datagram_received);
        let ret = res.unwrap();
        let size :usize = ret["SIZE"].clone().parse().unwrap();
        let jacked_msg = ret["MSG"].clone();
        let enc_msg = jacked_msg.split_at(size).0;
        let dec_msg_bytes = base64::decode(enc_msg).unwrap();
        let from = ret["DESTINATION"].clone();
        if !ret["ACK"].is_empty() {
            info!("ACK requested");
            let ack_type :u8 = ret["ACK"].clone().trim().parse().unwrap();
            match ack_type {
                0 => {
                    // Explicitly do not ack
                    info!("Do not ack")
                }
                1 => {
                    // Ack - don't wait on ack-ack
                    info!("Ack no wait")
                },
                2 => {
                    // Ack - wait on ack-ack and resend if not received within 90 seconds
                    info!("Ack and wait")
                },
                _ => {
                    // Assume do not ack although log that it's not supported
                    warn!("{} ack type not supported - ignoring with no ack", ack_type);
                }
            }
        }
        Ok((from, dec_msg_bytes))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SessionStyle {
    Datagram,
    Raw,
    Stream,
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

impl TryFrom<&str> for SessionStyle {
    type Error = ();
    fn try_from(original: &str) -> Result<Self, Self::Error> {
        match original {
            "DATAGRAM" => Ok(SessionStyle::Datagram),
            "RAW" => Ok(SessionStyle::Raw),
            "STREAM" => Ok(SessionStyle::Stream),
            n => Err(())
        }
    }
}

pub struct Session {
    sam: SamConnection,
    local_full_dest: String,
    local_dest: String,
    style: SessionStyle
}

impl Session {
    pub fn create<A: ToSocketAddrs>(
        sam_addr: A,
        destination: &str,
        nickname: &str,
        style: SessionStyle,
        min_version: &str,
        max_version: &str,
    ) -> Result<Session, Error> {
        let mut sam = SamConnection::connect(sam_addr, min_version, max_version)?;
        let create_session_msg = format!("SESSION CREATE STYLE={} ID={} DESTINATION={} \n", style.string(), nickname, destination);
        let ret = sam.send(create_session_msg, sam_session_status)?;
        let local_full_dest = ret["DESTINATION"].clone();
        info!("local_full_dest (size={}): {}",local_full_dest.len(),local_full_dest);
        let local_dest = sam.naming_lookup("ME")?;
        info!("local_dest (size={}): {}",local_dest.len(),local_dest);
        Ok(Session { sam, local_full_dest, local_dest, style })
    }

    pub fn sam_api(&self) -> io::Result<SocketAddr> {
        self.sam.conn.peer_addr()
    }

    pub fn naming_lookup(&mut self, name: &str) -> io::Result<String> {
        self.sam.naming_lookup(name)
    }

    pub fn duplicate(&self) -> io::Result<Session> {
        self.sam.duplicate().map( |s | Session {
            sam: s,
            local_full_dest: self.local_full_dest.clone(),
            local_dest: self.local_dest.clone(),
            style: SessionStyle::try_from(self.style.string()).unwrap()
        })
    }

    pub fn gen(&mut self, sig_type: SigType) -> Result<(String,String), Error> {
        self.sam.gen(sig_type)
    }

    pub fn send_msg(&mut self, to: String, msg: Vec<u8>, ack: u8) {
        self.sam.send_msg(to, msg, ack);
    }

    pub fn recv_msg(&mut self) -> Result<(String,Vec<u8>),Error> {
        self.sam.recv_msg()
    }

    // pub fn ping(&mut self, msg: &str) -> Option<String> {
    //     self.sam.ping(msg)
    // }

    // pub fn site(&mut self, host_dest: &str) -> Result<Vec<u8>, Error> {
    //     self.sam.site(host_dest)
    // }

    pub fn close(&mut self) {
        self.sam.conn.shutdown(Shutdown::Both).unwrap();
    }
}

pub struct StreamConnect {
    sam: SamConnection,
    session: Session,
    peer_dest: String,
    peer_port: u16,
    local_port: u16,
}

impl StreamConnect {
    pub fn new<A: ToSocketAddrs>(
        sam_addr: A,
        destination: &str,
        port: u16,
        nickname: &str,
        min_version: &str,
        max_version: &str,
    ) -> io::Result<StreamConnect> {
        let mut session = Session::create(sam_addr, "TRANSIENT", nickname, SessionStyle::Stream, min_version, max_version)?;
        let mut sam = SamConnection::connect(session.sam_api()?, min_version, max_version).unwrap();
        let create_stream_msg = format!("STREAM CONNECT ID={} DESTINATION={} SILENT=false TO_PORT={}\n", nickname, destination, port);
        sam.send(create_stream_msg, sam_stream_status)?;
        let peer_dest = session.naming_lookup(destination)?;
        Ok(StreamConnect { sam, session, peer_dest, peer_port: port, local_port: 0})
    }

    pub fn peer_addr(&self) -> io::Result<(String, u16)> {
        Ok((self.peer_dest.clone(), self.peer_port))
    }

    pub fn local_addr(&self) -> io::Result<(String, u16)> {
        Ok((self.session.local_full_dest.clone(), self.local_port))
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
    /// Destination used for establishing a Session (884 bytes): destination + priv key + signing key
    pub local_full_dest: String,
    /// Destination used for sending a Datagram (516 bytes): destination
    pub local_dest: String,
    session: Session
}

impl I2PClient {
    pub fn new(use_local: bool, alias: String, min_version: &str, max_version: &str, max_connection_attempts: u8) -> Result<I2PClient, Error> {
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

        let mut local_full_dest = String::new();
        let mut local_dest = String::new();
        let mut local_addr_loaded = false;

        if use_local {
            info!("i2p local dest file: {}", i2p_local_dest_path);

            if Path::new(i2p_local_dest_path).exists() {
                let mut i2p_local_dest_file = File::open(Path::new(i2p_local_dest_path)).unwrap();
                match i2p_local_dest_file.read_to_string(&mut local_full_dest) {
                    Ok(len) => {
                        if len > 0 {
                            local_addr_loaded = true;
                            info!("dest from file ({}): {}", len, &local_full_dest);
                        } else {
                            info!("{}","dest file empty");
                        }
                    },
                    Err(e) => warn!("{}", e)
                }
            }
        }
        if local_full_dest.is_empty() {
            // Establish Session, write to local_dest, and set dest
            match Session::create(DEFAULT_API,
                                  "TRANSIENT",
                                  alias.as_str(),
                                  SessionStyle::Datagram,
                                  min_version,
                                  max_version,
            ) {
                Ok(session) => {
                    local_full_dest = session.local_full_dest;
                    local_dest = session.local_dest;
                    if use_local && !local_addr_loaded {
                        info!("Saving dest to file: {}",i2p_local_dest_path);
                        match File::create(i2p_local_dest_path) {
                            Ok(f) => {
                                let mut d_file = f;
                                d_file.write_all(local_full_dest.clone().as_bytes()).unwrap();
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

        let mut attempts: u8 = 0;
        loop {
            info!("{}","Trying to create session...");
            let res = Session::create(DEFAULT_API,
                                      &local_full_dest.as_str(),
                                      &alias.as_str(),
                                      SessionStyle::Datagram,
                                      min_version,
                                      max_version);
            if res.is_ok() {
                info!("{}", "I2P Client initialized.");
                return Ok(I2PClient {
                    local_full_dest,
                    local_dest,
                    session: res.unwrap()
                })
            }
            attempts += 1;
            if attempts == max_connection_attempts {
                return Err(Error::new(ErrorKind::ConnectionRefused, format!("Unable to connect: max attempts ({}) reached", max_connection_attempts)))
            }
            warn!("Unable to create Session ({})...waiting a few seconds...", res.err().unwrap());
            thread::sleep(Duration::from_secs(3));
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

    /// Generate Public and Private keys IAW sig_type; return in tuple (PUB,PRIV)
    pub fn gen(&mut self, sig_type: SigType) -> Result<(String,String), Error> {
        self.session.gen(sig_type)
    }

    /// Send to destination UTF-8 formatted bytes
    pub fn send(&mut self, to: String, msg: Vec<u8>, ack: u8) {
        self.session.send_msg(to, msg, ack);
    }

    /// Receive tuple with from destination and message in UTF-8 formatted bytes
    pub fn receive(&mut self) -> Result<(String,Vec<u8>), Error> {
        self.session.recv_msg()
    }

    // pub fn ping(&mut self, msg: &str) -> Option<String> {
    //     self.session.ping(msg)
    // }

    // pub fn site(&mut self, host: &str) -> Result<Vec<u8>, Error> {
    //     let host_dest = I2PClient::dest(host);
    //     info!("Sending request for site: {}", &host_dest);
    //     self.session.site(host_dest.as_str())
    // }

    // pub fn shutdown(&mut self) {
    //     if self.session.is_some() {
    //         self.session.unwrap().close();
    //     }
    // }
}

// impl Service for I2PClient {
//     fn operate(&mut self, operation: u8, msg: Envelope) {
//         let mut packet = Packet::new(0, PacketType::Data as u8, NetworkId::I2P as u8, env.)
//         match operation {
//             1 => {
//
//             },
//             _ => {
//
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
