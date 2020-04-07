<div align="center">
  <img src="https://resolvingarchitecture.io/images/ra.png"  />

  <h1>Resolving Architecture</h1>

  <p>
    <strong>Clarity in Design</strong>
  </p>
  
  <h2>I2P Client</h2>
  
  <p>
   A client for a local I2P instance. 
   </p>
  
  <p>
    <a href="https://travis-ci.com/resolvingarchitecture/i2p-client"><img alt="build" src="https://img.shields.io/travis/resolvingarchitecture/i2p-client"/></a>
    <a href="https://crates.io/crates/i2p-client"><img alt="Crate Info" src="https://img.shields.io/crates/v/i2p-client.svg"/></a>
    <a href="https://docs.rs/crate/i2p-client/"><img alt="API Docs" src="https://img.shields.io/badge/docs.i2p-client-green"/></a>
  </p>
  <p>
    <a href="https://github.com/resolvingarchitecture/i2p-client/blob/master/LICENSE"><img alt="License" src="https://img.shields.io/github/license/resolvingarchitecture/i2p-client"/></a>
    <a href="https://resolvingarchitecture.io/ks/publickey.brian@resolvingarchitecture.io.asc"><img alt="PGP" src="https://img.shields.io/keybase/pgp/objectorange"/></a>
  </p>
  <p>
    <img alt="commits" src="https://img.shields.io/crates/d/i2p-client"/>
    <img alt="repo size" src="https://img.shields.io/github/repo-size/resolvingarchitecture/i2p-client"/>
  </p>
  <p>
    <img alt="num lang" src="https://img.shields.io/github/languages/count/resolvingarchitecture/i2p-client"/>
    <img alt="top lang" src="https://img.shields.io/github/languages/top/resolvingarchitecture/i2p-client"/>
    <a href="https://blog.rust-lang.org/2020/03/12/Rust-1.42.html"><img alt="Rustc Version 1.42+" src="https://img.shields.io/badge/rustc-1.42+-green.svg"/></a>
  </p>

  <h4>
    <a href="https://resolvingarchitecture.io">Info</a>
    <span> | </span>
    <a href="https://docs.rs/crate/i2p-client/">Docs</a>
    <span> | </span>
    <a href="https://github.com/resolvingarchitecture/i2p-client/blob/master/CHANGELOG.md">Changelog</a>
  </h4>
</div>

## Donate
Request BTC/XMR/ZEC address for a donation at brian@resolvingarchitecture.io.

## Notes
!! WIP - not stable until version 1.0 !!

## Goals

* Determine if local I2P client is installed using CLI
* Connect with local instance using CLI
* Send message over I2P using CLI
* Receive message over I2P using CLI
* Provide ability to launch as a network service and control its lifecycle
* Control local I2P instance using CLI
* Control local I2P instance as a service
* Support service_bus crate
* Support creating EEPSites


[Crates.io](https://crates.io/crates/i2p_client)

!! WIP - not stable until version 1.0 !!

## Setup
1. Download & Install I2P Router
2. Start I2P Router and wait 20 minutes to establish itself
3. Stop I2P Router
4. Enable SAMv3 port by changing parameter clientApp.0.startOnLoad from false to true in file 
01-net.i2p.sam.SAMBridge-clients.config located in directory: ~/.i2p/clients.config.d/