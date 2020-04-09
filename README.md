<div align="center">
  <img src="https://resolvingarchitecture.io/images/ra.png"  />

  <h1>Resolving Architecture</h1>

  <p>
    <strong>Clarity in Design</strong>
  </p>
  
  <h2>I2P Client</h2>
  
  <p>
   A client for a local I2P instance. Can be ran within the <a target="_blank" href="https://github.com/resolvingarchitecture/service-bus">Service Bus</a> as a Service.
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

*[x] Connect with local instance
*[x] Create minimal CLI
*[x] Lists known aliases
*[ ] Find a specific destination based on an alias
*[ ] Ping/Pong
*[ ] Send message over I2P
*[ ] Receive message over I2P
*[ ] Determine if local I2P router installed
*[ ] Determine if local I2P router running
*[ ] Control local I2P router instance lifecycle
*[ ] Support [service_bus](https://crates.io/crates/service-bus) crate implementing Service trait
*[ ] Support requesting EEPSite pages
*[ ] Support creating EEPSites

[Crates.io](https://crates.io/crates/i2p_client)

!! WIP - not stable until version 1.0 !!

## Setup - Ubuntu 18.04
1. Download & Install I2P Router
    ```shell script
    sudo apt-add-repository ppa:i2p-maintainers/i2p
    sudo apt-get update
    sudo apt-get install I2P
    ```
2. Start I2P Router from the commandline, wait for the [html console](http://127.0.0.1:7657/home) to launch, then wait for active peers to reach at least 10
    ```shell script
    i2prouter start
    ```
3. Stop I2P Router
    ```shell script
    i2prouter stop
    ```
4. Enable SAMv3 port by changing parameter clientApp.0.startOnLoad from false to true in file 
01-net.i2p.sam.SAMBridge-clients.config located in directory: ~/.i2p/clients.config.d/ (~ is your home directory, e.g. on Linux: /home/username)
5. Install Rust
   ```shell script
   sudo apt update
   sudo apt upgrade
   curl https://sh.rustup.rs -sSf | sh
   ```
6. Restart terminal
7. Verify Rust installed
    ```shell script
     rustc --version
    ```
8. Install build essentials
    ```shell script
    sudo apt install build-essential
    ```
