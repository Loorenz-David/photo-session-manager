# REEKON T1M Utility — source evidence

Started 2026-11-21 by the shaper for chapter `book/10I_measuring_tape.md`. Holds only verified facts with their
source and date. Nothing here is a protocol fact until the vendor's BLE guide/SDK (v1.5, October 2026, by request
from dev@reekon.tools) is received and its terms allow recording it (10I card MT5). **Do not add UUIDs, opcodes or
packet layouts before the terms are known.**

## Status
| Item | State |
|---|---|
| Hardware (T1M Utility, metric) | ordered by the owner, not arrived |
| BLE Connection Guide and SDK v1.5 | requested by the owner, not received |
| Licence / redistribution terms | unknown (card MT5) |

## Facts from the vendor's public pages (fetched 2026-11-21)
Product page `https://reekon.tools/products/t1m-utility-digital-tape-measure-tool`; Bluetooth page
`https://reekon.tools/pages/bluetooth-measuring-tools`.

| # | Fact | Source |
|---|---|---|
| F1 | A measurement is saved "with a single button press"; continuous streaming is not described. | product page |
| F2 | Bluetooth sharing to the vendor's REEKON ROCK and BOLDR apps; "Keyboard Emulation Mode"; "open software and connection options". | product page |
| F3 | Bluetooth 5.0. | product page |
| F4 | Accuracy "beyond 1/32 in" (0.5 mm in the FAQ). Display resolution not stated. | product page |
| F5 | Displays English and metric units; blades in English, metric or split units. | product page |
| F6 | "1000+ Measurement Memory" on the device (relevant to MC-MT1 `origin`: stored readings may be synced). | product page |
| F7 | Battery 150 h+ (FAQ: >100 h continual use), USB-C rechargeable; "Automatic Sleep Mode", timeout not stated. | product page |
| F8 | Keyboard mode types a measurement into any text field and can auto-advance spreadsheet cells; the exact typed string (unit suffix, Enter/Tab) is not stated. | Bluetooth page |
| F9 | An SDK/API/MCP access route exists and is linked from the Bluetooth page. | Bluetooth page |

The support articles cited by the owner's draft (BLE Connection Guide and SDK; "Is the T1M Bluetooth connected?")
returned HTTP 403 to automated fetching on 2026-11-21; read them in a browser and record facts here.

## Open questions (answer from the SDK or the vendor; each blocks the BLE adapter only)
| # | Question | Blocks |
|---|---|---|
| Q1 | How a press arrives: notification, indication, read, or command/response? | adapter design |
| Q2 | Service and characteristic UUIDs; packet layout, units, scale, endianness. | MC-MT1 decoding |
| Q3 | One event per press? Sequence number or id? | MC-MT1 `reading_id`, duplicate rule |
| Q4 | How stored memory is synced on connect, and whether synced readings are marked. | MC-MT1 `origin`, M-MT1 |
| Q5 | Connection behaviour on sleep, wake, power-off; who reconnects. | M-MT4 |
| Q6 | Bonding, pairing, authentication, firmware minimum, exclusive connection (does BOLDR on a phone block us?). | setup steps, status reasons |
| Q7 | Can a macOS CoreBluetooth client connect without the vendor app or cloud? | feasibility |
| Q8 | Licence, NDA, redistribution of the SDK and of protocol details in public source. | card MT5 |
| Q9 | Keyboard-mode output string (unit, decimal separator, terminator). | card MT4 |

## Hardware acceptance (M-MT5) — to record when the tape arrives
100 consecutive presses (count readings, compare each with the display); comparison against a conventional metric
tape on 10 lengths; sleep → wake; out of range → return; power-cycle; app restart; phone with BOLDR nearby. Record
observed counts and failures with date, firmware version and app build.
