# raspi demo for OLED ssd1351 display

https://www.waveshare.com/wiki/1.5inch_RGB_OLED_Module

## Quick start

```bash
> nix-shell
> rustup target add arm-unknown-linux-musleabihf
> cargo build --release --target arm-unknown-linux-musleabihf
> scp target/arm-unknown-linux-musleabihf/release/{display_all,display_off,refresh_json,take_measurement} 'pi@raspberrypi:~'
> # on the Pi, create sensors.db and events.json
> patchelf --set-interpreter /lib/ld-linux-armhf.so.3 display_all
> ./display_off on
> ./display_all sensors.db events.json temps
```

## Example

![temperature graph](./images/temps.png)

![events](./images/events.png)

(the second blue text is brighter on the OLED)
