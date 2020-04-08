extern crate dirs;
#[macro_use]
extern crate nom;

use std::fs::File;
use std::io::prelude::*;

use log::{debug,info,warn};

use std::clone::Clone;
use std::collections::HashMap;
use std::io;
use std::io::{BufReader, Error, ErrorKind, BufRead, Write, Read};
use std::path::{PathBuf, Path};
use std::net::{Shutdown, SocketAddr, TcpStream, ToSocketAddrs};

use nom::IResult;

mod parsers;
use crate::parsers::{gen_reply, sam_hello, sam_naming_reply, sam_session_status, sam_stream_status};

use ra_common::models::{Packet, Service, Envelope};
use i2p::sam::DEFAULT_API;
use ra_common::utils::wait::wait_a_ms;

static I2P_PID: &'static str = "i2p.pid";
static I2P_STATUS: &'static str = "i2p.status";

static SAM_MIN: &'static str = "3.0";
static SAM_MAX: &'static str = "3.1";

pub enum SigType {
    EDDSA_SHA512_ED25519,
    EDDSA_SHA512_ED25519PH,
    REDDSA_SHA512_ED25519
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
        let vec_opts = response.unwrap().1;
        verify_response(&vec_opts).map(|m| {
            m.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
    }

    fn handshake(&mut self) -> Result<HashMap<String, String>, Error> {
        let hello_msg = format!(
            "HELLO VERSION MIN={min} MAX={max} \n",
            min = SAM_MIN,
            max = SAM_MAX
        );
        self.send(hello_msg, sam_hello)
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

    pub fn gen(&mut self, sig_type: String) -> Result<String, Error> {
        let create_gen_msg = format!("DEST GENERATE SIGNATURE_TYPE={} \n", sig_type);
        let ret = self.send(create_gen_msg, gen_reply)?;
        Ok(ret["PUB"].clone())
    }

    pub fn duplicate(&self) -> io::Result<SamConnection> {
        self.conn.try_clone().map(|s| SamConnection { conn: s })
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
        Ok(Session {
            sam: sam,
            local_dest: local_dest,
        })
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
    pub local_dest: String
}

impl I2PClient {
    pub fn new(use_local: bool, alias: String) -> I2PClient {
        info!("{}", "Initializing I2P Client...");
        // Build paths
        let mut home = dirs::home_dir().unwrap();
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

        if use_local {
            info!("i2p local dest file: {}", i2p_local_dest_path);

            if Path::new(i2p_local_dest_path).exists() {
                let mut i2p_local_dest_file = File::open(Path::new(i2p_local_dest_path)).unwrap();
                match i2p_local_dest_file.read_to_string(&mut dest) {
                    Ok(len) => {
                        info!("dest ({}): {}", len, dest)
                    },
                    Err(e) => warn!("{}", e.to_string()),
                    _ => warn!("{}", "unable to load dest file")
                }
            } else {
                match File::create(Path::new(i2p_local_dest_path)) {
                    Ok(f) => info!("File for dest created: {}", i2p_local_dest_path),
                    Err(e) => warn!("{}", e.to_string()),
                    _ => warn!("{}", "unable to create dest file")
                }
            }
        }
        if dest.len() == 0 {
            // Establish Session, return dest, and write to local_dest
            match Session::create(DEFAULT_API,
                                  "TRANSIENT",
                                  alias.as_str(),
                                  SessionStyle::Datagram) {
                Ok(session) => {
                    info!("IP: {}, Dest: {}",session.sam_api().unwrap().ip().to_string(), session.local_dest);
                    dest = session.local_dest;
                    if use_local {
                        // Save
                        match File::open(Path::new(i2p_local_dest_path)).unwrap().write_all(dest.clone().as_bytes()) {
                            Ok(f) => info!("{} saved",i2p_local_dest_path),
                            Err(e) => warn!("{}", e.to_string()),
                            _ => warn!("unable to save dest file {}", i2p_local_dest_path)
                        }
                    }
                },
                Err(err) => {
                    warn!("Error: {}",err.to_string());
                }
            }
        }
        info!("{}","I2P Client initialized.");
        I2PClient {
            local_dest: dest
        }
    }

    // Handle incoming packets
    pub fn handle(&mut self, packet: &mut Packet) {
        info!("Handling incoming packet id={}",packet.id);

    }

    // Send out Packet with optional Envelope
    pub fn send(&mut self, packet: Packet) {
        match Session::create(DEFAULT_API,
                              packet.to_addr.as_str(),
                              "Anon",
                              SessionStyle::Datagram) {
            Ok(session) => {
                info!("IP: {}",session.sam_api().unwrap().ip().to_string())

            },
            Err(err) => {
                warn!("Error: {}",err.to_string());
            }
        }

        // let mut connection = StreamConnect::new(DEFAULT_STREAM_API, "1m5.i2p", 8000, "1m5").unwrap();
        // let local_addr = connection.local_addr().unwrap();
        // let peer_addr = connection.peer_addr().unwrap();
        // info!("Local addr: {}:{}",local_addr.0,local_addr.1);
        // info!("Peer addr: {}:{}",peer_addr.0,peer_addr.1);
    }
}

impl Service for I2PClient {
    fn operate(&mut self, operation: u8, env: Envelope) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
