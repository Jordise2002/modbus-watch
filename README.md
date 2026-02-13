# Ultrabus & Ultraslave
**Ultrabus and Ultraslave make talking to modbus devices easy!**

Ultrabus and Ultraslave are FOSS reproducible Modbus slave and master simulators designed to simplify the testing of Modbus devices and their integration with modern software solution

They expose a RESTful API that allows for the communication of any application with a Modbus device through http

Both Ultrabus and Ultraslave have all their configuration defined in a single JSON file allowing for easy reproduction of configurations like in, for example, an automatic testing scenario

## Usage

To run both programs you must specify the config file in the arguments. Like this
```bash
ultrabus config.json
```
or this
```bash
ultraslave config.json
```
All other optional parameters can be consulted by asking the program for help
```bash
ultrabus -h
ultraslave -h
```

## Documentation:

We are still working on the configuration description docs!

To find out about the Ultrabus API check the docs [here](https://jordise2002.github.io/Ultrabus/ultrabus.html)

To learn more about the Ultraslave API check the docs [here](https://jordise2002.github.io/Ultrabus/ultrabus.html)

## Installation

We still have to work on prebuilt releases, but you can always clone and compile this repo
```bash
git clone https://github.com/Jordise2002/Ultrabus
cargo build
```
## Modbus implementations support:

At the moment only Modbus TCP is supported, but support for Modbus RTU is also planned

## Compatibility

Ultrabus and Ultraslave can be compiled for any platform supported by Rust std
