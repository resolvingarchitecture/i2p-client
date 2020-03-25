[![Build Status](https://travis-ci.com/resolvingarchitecture/i2p-client.svg?branch=master)](https://travis-ci.com/resolvingarchitecture/i2p-client)
# I2P Client
A client for the local I2P instance. 

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