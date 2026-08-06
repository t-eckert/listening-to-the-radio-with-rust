# adsb-api

Serves the aircraft that [adsb-decoder](../adsb-decoder/) has tracked over HTTP as JSON, so
something other than a terminal can draw them — a web map, a phone, a slide.

Where [flight-tracker](../flight-tracker/) reads `adsb.db` and renders a TUI, this reads the
same database and answers HTTP requests. Both open it **read-only**; `adsb-decoder` remains
the only writer.

```
RTL-SDR → rtl_tcp → adsb-decoder → adsb.db ─┬→ flight-tracker  (TUI)
                                            └→ adsb-api        (HTTP/JSON)
```

## Run it

```bash
# In three terminals, or as services on the Pi
rtl_tcp
cargo run -p adsb-decoder
cargo run -p adsb-api
```

Then:

```bash
curl localhost:8080/api/aircraft | jq
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--db` | `adsb.db` | Database written by `adsb-decoder` |
| `--bind` | `0.0.0.0:8080` | Listen address |
| `--max-age-secs` | `1800` | Default window for `/api/aircraft` |
| `--bbox` | worldwide | `lat_min,lat_max,lon_min,lon_max` position filter |

## Endpoints

| Method | Path | Returns |
|--------|------|---------|
| `GET` | `/healthz` | Liveness plus the number of aircraft in the database |
| `GET` | `/api/aircraft` | Every aircraft seen recently, with its latest position |
| `GET` | `/api/aircraft/:icao/trail` | That aircraft's recent positions, oldest first |

`/api/aircraft` accepts `?max_age_secs=`, `/trail` accepts `?limit=` (default 200, capped at
10,000). ICAO addresses are case-insensitive. CORS is permissive, so a map served from
anywhere can call this directly.

```json
{
  "count": 68,
  "aircraft": [
    {
      "icao": "C07E33",
      "callsign": "ACA455",
      "altitude_ft": 10475,
      "ground_speed_kt": 252,
      "vertical_rate_fpm": 3136,
      "heading": 246.89,
      "lat": 45.407,
      "lon": -75.735,
      "messages": 178,
      "last_seen": "2026-03-22 19:06:36"
    }
  ]
}
```

## About `--bbox`

Position decoding uses CPR, which pairs an even and an odd message to recover a global
position. When the pairing is wrong, the result isn't noise — it's a *plausible-looking*
coordinate hundreds of kilometres away. Sorting by timestamp then gives you a trail that
teleports.

`--bbox` throws away positions outside a box you know the receiver can't hear. On real data
from Ottawa, one aircraft's trail looked like this without it:

```
-75.714, -75.717, -75.719, -82.492   <- the last point is in Lake Huron
```

and like this with `--bbox 44.0,46.5,-77.5,-74.0`:

```
-75.686, -75.700, -75.714, -75.717, -75.719
```

**The default is worldwide**, so the API works wherever the receiver is. Set a box to match
your location — a few hundred km around the antenna is plenty, since that's past the line-of-
sight range of 1090 MHz anyway.
