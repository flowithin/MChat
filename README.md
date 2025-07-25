<p align="center">
  <img src="assets/logo.png" alt="MChat logo" width="200">
</p>



---
# 🟦 MChat - Rust Chat App with AI Assistant


> A lightweight, terminal-based chat application built from scratch using Rust's `TcpStream`. Features a built-in AI assistant triggered via `@AI`.

---

## 🎥 Demo


<p align="center">
  <img src="assets/demo.gif" alt="MChat Demo" width="600">
</p>

---

## ✨ Features

- 🛠 Built using low-level Rust networking primitives (`TcpListener`, `TcpStream`)
- 💬 Real-time chat across multiple clients
- 🤖 AI Assistant: Mention `@AI` in your message to invoke an intelligent assistant
- ⚡ Blazing fast and memory-safe thanks to Rust
- 🎓 University of Michigan inspired design

---

## 🚀 Getting Started

### Prerequisites

- Rust (>=1.70)
- Cargo

### Installation

```bash
git clone https://github.com/yourusername/mchat.git
cd mchat
cargo build --release
````

---

## 🖥️ Usage

### Start the server

```bash
cargo run --bin server
```

### Start a client

In a new terminal:

```bash
cargo run --bin client
```

Connect multiple clients in different terminals to simulate multi-user chat.

---

## 🧠 Using the AI Assistant

To invoke the AI assistant, simply type a message like:

```
@AI What's the weather today?
```

The assistant will respond directly in the chat with relevant information.

---

## 🏫 Inspired By

This project was built with ❤️ at the University of Michigan. Go Blue! 💙💛
Logo courtesy of [Wikipedia Commons](https://en.wikipedia.org/wiki/File:Michigan_Wolverines_logo.svg).

---

## 📄 License

MIT License

---

## 🤝 Contributions

Pull requests are welcome! For major changes, please open an issue first to discuss what you would like to change.

---

## 📬 Contact

Feel free to reach out via issues or email me at [jtwuuuuu@umich.edu](mailto:your-email@umich.edu).

---

Let me know if you'd like:

- A **custom UMich-style logo** for MChat (I can generate one for you).
- Embedded YouTube demo markdown if you're hosting it there.
- The `Cargo.toml` file and module layout to match this documentation.

