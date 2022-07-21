# Enabling deep sleep for your Raspberry Pi project

Single board computers (SBCs) are awesome, small, low-power devices, often with wireless connectivity, which makes them ideal for simple **battery operated projects**.

However, when calculating your energy budget, you often come to realize they are not quite low power enough to accommodate for small batteries, or long-time (multiple days/months) operation. We often don't need full-time availability, so why not **putting our computer to sleep to improve our project battery life**?

In this article, we will discuss a **simple approach to enable deep-sleep capability** on your next battery powered Raspberry Pi (or any SBC-powered) project.

## The study case

During this article, we will use our BalenaLabs [Inkyshot](https://github.com/balenalabs/inkyshot) project, which perfectly fits the intermittent operation, battery operable use case.

![](https://raw.githubusercontent.com/balenalabs-incubator/inkyshot/master/assets/header-photo.jpg)

To put it in few words, the project at startup will *connect to the internet, pull a new inspirational quote, or your local weather, and then will update the eInk display*, this process is repeated once in a while (generally once every hour), staying idle the rest of the time.

This project is perfect for battery operation because *eInk display retains the display without power*, which allows us to power down the system entirely to save battery.

## Bill of materials

To reproduce this project, you will need:
- A Raspberry Pi compatible SBC,
- A SD card,
- A [Raspberry Pi Pico](https://www.raspberrypi.com/products/raspberry-pi-pico/) micro-controller board,
- A voltage regulator with enable pin (we used _Weewooday-5825_ found on Amazon, but you can also [convert a LM2596S kit](./control_lm2596s_enable.md)),
- A [Inky pHat](https://shop.pimoroni.com/products/inky-phat?variant=12549254217811) or [Waveshare 2.13"](https://www.waveshare.com/2.13inch-e-paper-hat.htm) eInk display module,
- 2 USB-A to Micro-USB cables (or one + one A to C if using a Pi4),
- A power source in the 6-12V range, with at least 1A capability (depending on the Raspberry you chose),
- A way to connect to your power source (alligator clips, barrel jack...),
- A perf board to mount everything together (optional).

You will also need:
- A soldering iron,
- some solder,
- some wires to connect everything.

## Our goal

To keep power consumption as low as possible, the best approach is to:

1. Start our system,
2. <span style="color:gray">_Connect to the internet (optional, depending on the project requirement)_</span>,
3. <span style="color:gray">_Load our project state from storage (optional, depending on the project requirement)_</span>,
4. Do our project processing, updates, communications...,
5. Program our next wake up conditions,
6. Go to deep sleep to preserve as much power as possible,
7. Repeat from [2] on wake-up.

**TODO: Add a state machine visual representation of the process.**

If we apply it to our use-case, we want to:

1. Start our system at power-up,
2. Connect to the internet,
3. Pull our data and update the display,
4. Program a 1h sleep period,
5. Go into deep sleep,
6. Repeat from [1].

### The problem

Doing such an operation cycle is however _not natively supported on RaspberryPies_, as they can't natively go to low power mode. The only way to drastically reduce power consumption is to _shut down the SBC and cut down the power supply_.

Once the system is shut down, you need manual/external intervention to power it back on.

## Proposed solution: Morpheus

To reduce the power consumption, a common approach in embedded systems is to cut the power supply on the main energy hungry components. To do so, we will use an ultra low power external circuit able to keep track of the time, communicate with the Raspberry Pi, and control its input power supply.

Here enters _Morpheus_.

### Presentation

_Morpheus_ (named after the [greek god of sleep and dreams](https://en.wikipedia.org/wiki/Morpheus)), is a Raspberry Pi Pico based project designed to interface between your Raspberry Pi power supply and your Raspberry Pi. 

### Architecture

### Usage

#### 1. Hardware requirements

#### 2. Setting up the MCU

#### 3. Adding the block to your project

#### 4. Using the API

#### 5. Using the CLI script


## Discussion

Consideration for project development:
- The system state is lost on every power down, you should load/save the state on disk (or other method)
- Keep the system running while updates are pending


### Improvements

- Putting the Pico in deep sleep to preserve even more power
- Shutting down Pico USB while the Pi is not powered
- Adding other sources of wake up (I2C sensor threshold value...)
- Using other low power MCU to do the deep sleep with other capabilities (Pico W, ESP32C3...)