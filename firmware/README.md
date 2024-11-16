# Firmware

## MCU

- ESP32

## Wire Connection

| ESP32   | E-Paper |
| ------- | ------- |
| GPIO 13 | CLK     |
| GPIO 14 | DIN     |
| GPIO 25 | BUSY    |
| GPIO 26 | RST     |
| GPIO 15 | CS      |
| GPIO 27 | DC      |
| GPIO 23 | PWR     |

## Design

### Mode

| Mode       | Description                                                  |
| ---------- | ------------------------------------------------------------ |
| Initialize | Guides the user through the initial setup.                   |
| Settings   | The device stays connected to the Internet and users can connect to the device for setup. |
| Normal     | The regular state of the device, refreshing the information on the screen at regular intervals. Most of the time it is hibernating. |
| Low Power  | The device no longer refreshes information and displays a low-battery alert on the screen. |

### Function Key

The key on the back is used as a function key.

#### Initialize Mode

None

#### Settings Mode

| Operation | Behavior            | Alert |
| --------- | ------------------- | ----- |
| Press     | Leave settings mode |       |

#### Normal Mode

| Operation           | Behavior            | Alert        |
| ------------------- | ------------------- | ------------ |
| Press               | Refresh home page   |              |
| Press & hold 3 secs | Enter settings mode |              |
| Press & hold 7 secs | Reset device        | LED blinking |

#### Low Power Mode

| Operation | Behavior                                                     | Alert |
| --------- | ------------------------------------------------------------ | ----- |
| Press     | Re-check the device's power level and return to normal mode when the power level is sufficient |       |
