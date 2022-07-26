# Morpheus-block REST API

The Morpheus-Block exposes a HTTP REST API on its TCP port 5555.

This documents exposes available commands, parameters and response format, and proposes CURL based requests to query the API.

## GET /version

This endpoint can be used to query the Morpheus Firmware version, which might be useful to check for certain functions availability.

### Versions

This function is available from version `0.1.0`.

### Input Parameters

This call doesn't take input parameters

### Response
The response is a JSON containing the version as `major`, `minor` and `patch` index:
```
{"GetVersion":{"major":0, "minor":1, "patch":0}}
```

### Example call
You can request the version with CURL:

```
curl -sX GET "http://morpheus-serial:5555/version"
```



## GET /sleep_pin/PRE/LEVEL

This endpoint can be used to request a sleep action with sensitivity on the wake-up pin.
The Pi will be put to sleep after a `PRE` seconds delay, and will wake up when GPIO 3 is set to `LEVEL`.

### Versions

This function is available from version `0.1.0`.

### Input Parameters

This call takes two parameters passed in the URL as `PRE` and `LEVEL`:
- `PRE` is the duration of the 'pre-sleep' delay, it is a _NUMBER_ in seconds (_e.g._ 90),
- `LEVEL` is the GPIO pin level on which the Pi will wake up. This should be a _BOOLEAN_ value (`true` or `false`).

### Response
The response is an acknowledgement of the command:
```
{"SleepPin":{"success":true}}
```

### Example call
You can request the version with CURL, this will put the Pi in sleep after `90` seconds, and wait for GPIO3 to be connected to GND to wake up:

```
curl -sX GET "http://morpheus-serial:5555/sleep_pin/90/false"
```

## GET /sleep_time/PRE/DELAY

This endpoint can be used to request a sleep action with a fixed sleep period.
The Pi will be put to sleep after a `PRE` seconds delay, and will wake up after `DELAY` seconds.

### Versions

This function is available from version `0.1.0`.

### Input Parameters

This call takes two parameters passed in the URL as `PRE` and `LEVEL`:
- `PRE` is the duration of the 'pre-sleep' delay, it is a _NUMBER_ in seconds (_e.g._ 90),
- `DELAY` is the duration of the 'sleep' delay, it is a _NUMBER_ in seconds (_e.g._ 1800).


### Response
The response is an acknowledgement of the command:
```
{"SleepTime":{"feedback":1}}
```

### Example call
You can request the version with CURL, this will put the Pi in sleep after `90` seconds, and wait for 60min before waking up:

```
curl -sX GET "http://morpheus-serial:5555/sleep_time/90/3600"
```

