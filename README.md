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
    <a href="https://docs.rs/crate/i2p_client/"><img alt="API Docs" src="https://img.shields.io/badge/docs.i2p-client-green"/></a>
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

[I2P-RS version](https://github.com/i2p/i2p-rs) use attempted but not fully working as of 1st Qtr 2020.

## Goals

*[ ] 1.0.0 - Full SAMv1.0 Compliance
    *[x] 0.1.0 - Minimal CLI
        *[x] Connect with local instance
        *[x] Lists known aliases with b32 addresses (from published personal addressbook - .i2p/eepsite/docroot/hosts.txt)
        *[x] Find a specific destination based on an alias
        *[x] Create minimal CLI
    
    *[ ] 0.2.0 - Basic I/O
        *[ ] Send message over I2P
        *[ ] Receive message over I2P
    
    *[ ] 0.3.0 - Service Bus Support
        *[ ] Support [service_bus](https://crates.io/crates/service-bus) crate implementing Service trait
    
    *[ ] 0.4.0 - EEP Site Support
        *[ ] Support requesting EEPSite pages persisting locally (started but unsure how to make request/reply using SAM interface)
    
    *[ ] 0.5.0 - Examples
        *[ ] Example CLI use cases
        *[ ] Example Service use cases
    
    *[ ] 0.6.0 - Fully Documented
        *[ ] README.md completed
        *[ ] All code documented
        *[ ] All examples documented
    
    *[ ] 0.7.0 - Fully Tested and Released for Production
    
    *[ ] 0.8.0 - Ease of installation
        *[ ] Determine if local I2P router installed
        *[ ] Determine local I2P router status
        *[ ] Auto-install I2P router
    
    *[ ] 0.9.0 - All SAMv1.0 features implemented and tested

*[ ] 2.0.0 - Full SAMv2.0 Compliance

*[ ] 3.0.0 - Full SAMv3.0 Compliance

*[ ] 3.1.0 - Full SAMv3.1 Compliance

*[ ] 3.2.0 - Full SAMv3.2 Compliance
    *[ ] Ping/Pong (Code implemented; waiting on SAM 3.2 support)

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
01-net.i2p.sam.SAMBridge-clients.config located in your home directory at: .i2p/clients.config.d/ (your home directory on Linux: /home/username)
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
