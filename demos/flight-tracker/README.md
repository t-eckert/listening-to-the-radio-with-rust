# flight-tracker

A terminal UI that displays aircraft on a map of the Ottawa area, using data from the [adsb-decoder](../adsb-decoder/).

## Run it

```bash
# First, run adsb-decoder in another terminal to populate the database
cargo run -p adsb-decoder

# Then start the flight tracker
cargo run -p flight-tracker
```

Press `q` to quit. Use arrow keys to navigate the aircraft list.

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--db` | `adsb.db` | Path to the SQLite database |
| `--refresh` | `1000` | Refresh interval in ms |

## What you'll see

- **Left panel**: a map of the Ottawa area with aircraft positions plotted as dots, flight trails, and landmarks (Parliament, YOW airport, rivers)
- **Right panel**: a table of all tracked aircraft with callsign, altitude, speed, heading, and message count

Aircraft appear as the adsb-decoder picks up their transmissions. Over a few minutes, you'll see flight paths forming as planes move across the map.

## No SDR hardware needed (directly)

The flight tracker reads from a SQLite database, not from the SDR directly. You need the adsb-decoder running to populate that database, but the tracker itself is just a visualization tool.
