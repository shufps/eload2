> ⚠️ **Disclaimer / Status**
>
> This project is provided **AS IS**, **without any warranty** and **without any kind of support**.
> Use at your own risk — this is **high-current / high-power** hardware and can damage equipment or cause injury if used incorrectly.

# eload2 (Load v2)

<img width="800" alt="image" src="https://github.com/user-attachments/assets/509d11a5-d444-4df1-8f3a-e6a2a1d3adae" />

<img width="800" alt="image" src="https://github.com/user-attachments/assets/fe027fa6-4e62-4210-81a0-b0daeddadc44" />


A compact, over-engineered **multi-channel electronic load** for **low-voltage / high-current** testing — built because nothing affordable exists that does *0–1.5V @ ~100A* properly.

No display. Control via **USB** from a PC (Python), firmware on **STM32**.

## Why this exists

I needed a tool to:
- test **buck converters / VRMs** in the **0–1.5V** range at serious currents
- still be usable as a “normal” load at **12V** (e.g. for power supplies / battery projects)
- be stable, repeatable and boring (in the good sense)

Commercial options in this niche are either impractical or unreasonably expensive — so I built it.

## Highlights

- **4 independent channels**
- Designed for:
  - **0–1.5V** VRM / buck testing up to **~100A** total (depending on cooling & SOA)
  - **12V** testing (e.g. ~8–12A, depending on dissipation)
- Each channel has its own analog supply chain (low coupling, predictable behavior):
  - dedicated **9V LDO**
  - dedicated precision **reference** (REF3433)
  - dedicated **16-bit DAC**
  - dedicated **OPA2196** control loop
  - dedicated MOSFET “arm”
- MOSFETs: **BUK7S0R5-40HJ** (very low Rds(on), beefy SOA)
- Cooling: **SP3/TR4 CPU cooler** (tested in practice)
- High-current IO: **M4** power connections (aiming for ~80A per connector)
- PCB: **120 × 160 mm**, **4-layer**, 1oz/1oz
  (large copper, parallel layers, big via arrays; optional copper busbars)

## Status

- PCBs + stencil: ✅
- Parts: ✅ (BOM originally ~1.5 years old, most parts still available)
- Bring-up: ✅
- Verified load test: **12V @ 10A (120W) for ~15 min → ~55°C board temperature** (with the big cooler)

## Errata

- Current setting is a little bit off. For example 11.5A needs to be set for 10.0A current.

## Minimalistic Web UI

There is a minimalistic web UI.

<img width="800" alt="image" src="https://github.com/user-attachments/assets/77e32ae6-fa66-4057-a8ec-0efb4c8a2999" />

It can be run using `streamlit`:
```bash
streamlit run eload_ui.py
```


## Documentation

https://www.mikrocontroller.net/topic/563073

## Repository structure (typical)

- `kicad/` – schematics, PCB, mechanical notes
- `firmware/` – STM32 firmware (Rust, Embassy)
- `python/` – PC tooling / scripts (Python)

## Quick start

### Firmware
- Build/flash using your usual STM32 + Rust toolchain.
- This project uses **Embassy** async tasks.

### Host control
- Connect the board via USB.
- Use the Python scripts to set:
  - per-channel DAC setpoints
  - PWM / fan / enable (depending on build)
  - read back telemetry (if enabled)

## Safety / Notes

This is a **high current** design.
- Use short, thick wiring for low-voltage/high-current tests.
- Treat every milliohm as a heater.
- Ensure solid cooling and stable mounting before sustained high power tests.
- If you don’t know what you’re doing: don’t connect this to expensive hardware.

## License

CERN-OHL-S: https://ohwr.org/cern_ohl_s_v2.pdf


