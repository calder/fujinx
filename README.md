<p align="center">
  <img alt="Fujinx" width="300" src="https://github.com/calder/fujinx/blob/main/src/fujinx.png?raw=true">
  <br>
  <em>command line tools for <a href="https://www.fujifilm-x.com/en-us/products/x-series/">Fujifilm X Series</a> cameras</em>
</p>

## 📦 Install

Install from [Crates.io](https://crates.io/):
```sh
cargo install --locked fujinx
```

Install from source:
```sh
cargo install --locked --path .
```

## 🛠️ Usage

```sh
fj                                                      # Show help
fj camera list                                          # Show connected cameras
fj convert --recipe=pacific_blues DSCF2930.raf          # Convert RAW to JPEG

fj recipe show -c1                                      # Show recipe in camera slot 1
fj recipe save -c1 test                                 # Save recipe from camera slot 1
fj recipe load -c1 test                                 # Load recipe into camera slot 1
fj recipe list                                          # List available recipes

fj repo add https://github.com/calder/fujixweekly       # Add community recipe repo
fj repo update                                          # Update community recipe repos
fj repo list                                            # List community recipe repos
```

## ✨ Features

### 🖥️ Operating Systems

|         | Support | Maintainer | Transport |
| ------- | ------- | ---------- | --------- |
| Linux   | ✅ | [calder](https://github.com/calder) | [rusb](https://github.com/a1ien/rusb) + [libusb](https://libusb.info/) |
| Mac     | ✅ | [calder](https://github.com/calder) | [rusb](https://github.com/a1ien/rusb) + [libusb](https://libusb.info/) |
| Windows | ❌ | wanted | |

### 📷 Cameras

| Camera | Maintainer | [▦](https://www.fujifilm-x.com/en-us/products/x-series/ "Sensor") | [🖼️](src/cmd/convert.rs "RAW Conversion") | [⚙️](src/cmd/config.rs "Configuration Management") | [📷](. "Tethered Shooting") |
|-|-|-|-|-|-|
| [X-M5](https://www.fujifilm-x.com/en-us/products/cameras/x-m5/) | [calder](https://github.com/calder) | [4️⃣](https://www.fujifilm-x.com/en-us/products/cameras/x-m5/specifications/ "26.1MP X-Trans CMOS 4") | ✅ | ✅ | ❌ |
| [X-T30 III](https://www.fujifilm-x.com/en-us/products/cameras/x-t30-iii/) | wanted | [4️⃣](https://www.fujifilm-x.com/en-us/products/cameras/x-t30-iii/specifications/ "26.1MP X-Trans CMOS 4") | ❌ | ❌ | ❌ |
| [X-E5](https://www.fujifilm-x.com/en-us/products/cameras/x-e5/) | wanted | [5️⃣ HR](https://www.fujifilm-x.com/en-us/products/cameras/x-e5/specifications/ "40.2MP X-Trans CMOS 5 HR") | ❌ | ❌ | ❌ |
| [X-S20](https://www.fujifilm-x.com/en-us/products/cameras/x-s20/) | wanted | [4️⃣](https://www.fujifilm-x.com/en-us/products/cameras/x-s20/specifications/ "26.1MP X-Trans CMOS 4") | ❌ | ❌ | ❌ |
| [X-T50](https://www.fujifilm-x.com/en-us/products/cameras/x-t50/) | wanted | [5️⃣ HR](https://www.fujifilm-x.com/en-us/products/cameras/x-t50/specifications/ "40.2MP X-Trans CMOS 5 HR") | ❌ | ❌ | ❌ |
| [X-T5](https://www.fujifilm-x.com/en-us/products/cameras/x-t5/) | wanted | [5️⃣ HR](https://www.fujifilm-x.com/en-us/products/cameras/x-t5/specifications/ "40.2MP X-Trans CMOS 5 HR") | ❌ | ❌ | ❌ |
| [X100VI](https://www.fujifilm-x.com/en-us/products/cameras/x100vi/) | wanted | [5️⃣ HR](https://www.fujifilm-x.com/en-us/products/cameras/x100vi/specifications/ "40.2MP X-Trans CMOS 5 HR") | ❌ | ❌ | ❌ |
| [X-H2](https://www.fujifilm-x.com/en-us/products/cameras/x-h2/) | wanted | [5️⃣ HR](https://www.fujifilm-x.com/en-us/products/cameras/x-h2/specifications/ "40.2MP X-Trans CMOS 5 HR") | ❌ | ❌ | ❌ |
| [X-H2s](https://www.fujifilm-x.com/en-us/products/cameras/x-h2s/) | wanted | [5️⃣ HS](https://www.fujifilm-x.com/en-us/products/cameras/x-h2s/specifications/ "26.1MP X-Trans CMOS 5 HS") | ❌ | ❌ | ❌ |
