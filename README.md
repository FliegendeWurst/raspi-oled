# raspi demo for OLED ssd1351 display

## Quick start

```bash
> nix-shell
> rustup target add arm-unknown-linux-musleabihf
> cargo build --release --target arm-unknown-linux-musleabihf
> scp target/arm-unknown-linux-musleabihf/release/{display_all,display_off,refresh_json,take_measurement} 'pi@raspberrypi:~'
> # on the Pi:
> patchelf --set-interpreter /lib/ld-linux-armhf.so.3 display_all
> ./display_off on
> ./display_all sensors.db events.json temps
```

## Example

![picture](./images/temps.png)

![primitive](./images/events.png)
