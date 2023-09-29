# SM2335EGH-rs

A GPIO-based driver for the SM2335EGH LED controller used in the SwitchBot Color Bulb, written in pure no-std Rust.

The SM2335EGH (aka just SM2335) is a 5-channel, 10-bit LED controller made by Shenzen Sunmoon Microelectronics.
[Some details about the chip can be found on their website, chinaasic.com](http://www.chinaasic.com/chipDetails/detail_290.html).
Alternatively, you can find the [document with the specs here in doc/](doc/SM2335EGH-chip-details-ch.pdf).

In short, the five channels (called OUT1 through OUT5) are essentially split into two groups.
The first three channels (OUT1-OUT3) are low voltage at 40V and allow a maximimum current of 160mA.
In practice, these three channels are used for RGB / coloured light.
The last two channels (OUT4-OUT5) are much higher voltage at 500V, but the maximum current is halved at 80mA.
These two channels are used for warm & cool white.

In my SwitchBot bulbs, this is the concrete channel mapping used:

| Output | Group | Color/hue  |
|--------|-------|------------|
| OUT1   | RGB   | Green      |
| OUT2   | RGB   | Red        |
| OUT3   | RGB   | Blue       |
| OUT4   | CW    | Warm white |
| OUT5   | CW    | Cold white |

This chip seems fairly uncommon at this point (2023-09).
At least I'm not aware of any other products than the SwitchBot bulbs that use it.
The specific model number of my bulbs with this chip is W1401400.

![Rainbow color cycle on a dismantled SwitchBot Color Bulb connected to a flasher/debugging probe](https://media.giphy.com/media/vFKqnCdLPNOKc/giphy.gif)

## Implementation

I just based the driver on the ones in [Tasmota](https://github.com/arendst/Tasmota/pull/15839) and [ESPHome](https://github.com/esphome/esphome/pull/3924). I've asked the manufacturer for more information about the protocol, just to have a first hand source, but I'm not particularly worried about bugs. The protocol as found in the Tasmota and ESPHome implementations is really simple, and I haven't had any issues in practice. 

However, if you happen to have access to the protocol specification, please contact me! Similarly, if you've spotted an issue in the current implementation -- don't hesitate to open an issue (or even better, a PR).

## License

The MIT License (MIT). See [LICENSE](LICENSE).
