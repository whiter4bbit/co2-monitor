# co2-monitor

Monitor that can be started on RPI. It periodically reads measurements from the [device](http://www.wetterladen.de/aircontrol-co2-monitor-mini-tfa-31.5006) and sends results to the provided URL in [csv format](src/measurement.rs).

The project was inspired by the [Reverse-Engineering a low-cost USB CO₂ monitor](https://hackaday.io/project/5301-reverse-engineering-a-low-cost-usb-co-monitor) project.

## Building on RPI

(Tested on 'Raspbian GNU/Linux 8 (jessie)')

* Install [rustup](https://rustup.rs)
* Install required libraries `sudo apt-get install -y libusb-1.0.0-dev libssl-dev`

```
git clone git@github.com:whiter4bbit/co2-monitor.git
cd co2-monitor
cargo build
```
