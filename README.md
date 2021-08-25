# mode-s

## Example
```
# Startup dump1090-fa (https://github.com/flightaware/dump1090.git)
./dump1090 --net --quiet

# Startup 1090 decode chain using this library
cargo r --example 1090
```

## Radar tui

TODO: Add image
```
# Startup dump1090-fa (https://github.com/flightaware/dump1090.git)
./dump1090 --net --quiet

# Startup "radar" display in tui relative to your sdr position
cargo r --example radar -- --lat="50.0" --long="50.0"
```
