# 📘 Bare Metal Rasberry Pi

> Implement bare-metal (no std library) C++ and Rust system controllers on a resberry pi pico

---

## 🧩 Overview

Embedded systems form the backbone of modern electronics — from IoT to robotics.
They operate under tight memory and timing constraints, prioritizing reliability and control over raw convenience.
This project develops a stepper motor controller on the Raspberry Pi Pico 2, implemented twice:
•	Once in C++ using the Raspberry Pi SDK.
•	Once in Rust using the rp-hal embedded hardware abstraction library.

---

## 🎯 Objectives / Learning Goals

- 🔹 [Goal 1] — Understand bare-metal programming (no standard library).  
- 🔹 [Goal 2] — Learn GPIO, PWM, and interrupt control at the register level  
- 🔹 [Goal 3] — Compare safety, readability, and performance between Rust and C++ 

---

## ⚙️ Tech Stack

| Category | Tools / Languages |
|-----------|------------------|
| Language | `C/C++`, `Rust` |
| Frameworks | `rp-hal`, `embedded-hal` |
| Tools | `Raspberry Pi Pico SDK` |
| Hardware | Raspberry Pi Pico, Stepper Motor, etc. |

---

## 🚀 Features / Implementation Plan

1. **Step 1 – Setup:** brief description (e.g., initialize repo, install deps)  
2. **Step 2 – Core Functionality:** what’s being built and how  
3. **Step 3 – Comparison / Benchmarking:** if relevant  
4. **Step 4 – Visualization / Output:** how results are displayed or tested  
5. **Step 5 – Deployment / Packaging:** optional (web, CLI, hardware, etc.)

> 📈 You can add diagrams or screenshots here  
> `![screenshot](docs/screenshot.png)`  

---

## 🧪 Usage / Running the Project

### 🖥️ Setup

This is a rust project so installing [Rust](https://rustup.rs/) is essential. This can be found at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

The project is run on a pi pico 2W which means the more developed/stable rp-2040 crate is not applicable and will cause errors. The pi pico 2 has an update processor, the RP2350 which requires the rp235x crate.
Secondly, the wifi chips require the cyw43 crate to use the on-board systems (Bluetooth, Wireless **AND the on-board LED**). The cyw43 driver is async-only, so you need the embassy async executor. The embassy allows for async/await programming in rust.


```bash
cargo install cargo-generate
```


```bash
# Clone the repository
git clone https://github.com/<yourusername>/<project-name>.git
cd <project-name>

# Install dependencies
cargo build
```

### ▶️ Run

```bash
# Example run command
npm run dev        # or python main.py / go run main.go / cargo run
```

---

## 📊 Results / Observations

- Key metrics or outcomes  
- Screenshots or performance graphs  
- Lessons learned, trade-offs, or insights  

---

## 🔮 Future Improvements

- [ ] Feature idea 1  
- [ ] Feature idea 2  
- [ ] Optional enhancements, refactors, or optimizations  

---

## 📚 References / Resources

- [The Rust Book](https://doc.rust-lang.org/book/print.html)
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/print.html)

- [Embassy example for rp235x](https://github.com/embassy-rs/embassy/tree/main/examples/rp235x)
- [Template for rp235x](https://github.com/rp-rs/rp235x-project-template/tree/main)

- [Raspberry Pi Pico SDK](https://github.com/raspberrypi/pico-sdk-tools/releases)

- [Pi Pico 2W Datasheet](https://datasheets.raspberrypi.com/picow/pico-2-w-datasheet.pdf)

---

## 🧑‍💻 Author

**Olly Bayley**  
GitHub: [@ombayley](https://github.com/ombayley)  

---

## 🪪 License

This project is licensed under the **GNU General Public License (GPL)** — See the [LICENSE](LICENSE) file for details.
The GPL License is a copyleft license, that requires any derivative work to also be released under the GPL License.
This means any derivative software that uses this code remains open-source and freely available to the public.
