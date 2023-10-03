# Raspberry Pi calendar/temperature monitoring station

For more details see https://fliegendewurst.github.io/raspberry-pi-temperature-monitoring.html

The used OLED display is from [Waveshare](https://www.waveshare.com/wiki/1.5inch_RGB_OLED_Module)

## Quick start

```bash
> nix-shell
> rustup target add arm-unknown-linux-musleabihf
> cargo build --release --target arm-unknown-linux-musleabihf
> scp target/arm-unknown-linux-musleabihf/release/{display_all,display_off,refresh_json,take_measurement,status_check_example} 'pi@raspberrypi:~'
> # on the Pi, create sensors.db and events.json
> ./status_check_example > /run/user/1000/status.json
> patchelf --set-interpreter /lib/ld-linux-armhf.so.3 display_all
> ./display_off on
> ./display_all sensors.db events.json temps
```

## Example

![temperature graph](./images/temps.png)

![events](./images/events.png)

(the blue text seen in the second image is bright enough on the real OLED display)

## License

Copyright 🄯 2022-2023 FliegendeWurst

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

See [`LICENSE`]. Applies to all files except the ones listed below.

`src/rpi.raw` is the Raspberry Pi logo. Raspberry Pi is a trademark of Raspberry Pi Ltd.