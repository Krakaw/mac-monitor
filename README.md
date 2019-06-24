<h1 align="center">Welcome to mac-monitor 👋</h1>
<p>
  <img src="https://img.shields.io/badge/version-0.1.0-blue.svg?cacheSeconds=2592000" />
  <a href="https://github.com/Krakaw/mac-monitor/blob/master/LICENSE">
    <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-yellow.svg" target="_blank" />
  </a>
</p>

> Get notifications when mac addresses change on your network.
> After first load a snapshot is taken, any devices added can trigger a notification.
> Any devices removed can trigger a notification. 
> Or specific mac address changes can trigger a notification.

## Install

```sh
cargo install mac-monitor
```

## Usage


```sh
# List interfaces
mac-monitor -l
```

```sh
# Show all MAC addresses on the network
mac-monitor -i en0
```

```sh
# Start monitoring for these specific MAC addresses
mac-monitor -i en0 -m de:ad:be:ef:00:00 aa:bb:de:ad:be:ef
```

## Author

👤 **Krakaw**

* Github: [@Krakaw](https://github.com/Krakaw)

## 🤝 Contributing

Contributions, issues and feature requests are welcome!<br />Feel free to check [issues page](https://github.com/Krakaw/mac-monitor/issues).


## 📝 License

Copyright © 2019 [Krakaw](https://github.com/Krakaw).<br />
This project is [MIT](https://github.com/Krakaw/mac-monitor/blob/master/LICENSE) licensed.