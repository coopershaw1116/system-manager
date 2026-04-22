# Development Stack

This project is a high-performance, native system monitor built for the modern Linux ecosystem (2026). It prioritizes memory safety, adaptive UI, and asynchronous system communication.

## 🛠 The Core Stack

| Component | Technology | Role |
| :--- | :--- | :--- |
| **Language** | **Rust** | Provides memory safety and high-speed execution without a garbage collector. |
| **Framework** | **Relm4** | An idiomatic GUI library for Rust inspired by the Elm Architecture (Model-View-Update). |
| **Toolkit** | **GTK4 / Libadwaita** | The industry standard for modern, hardware-accelerated Linux desktop applications. |
| **Declarative UI** | **Blueprint** | A human-readable markup language used to design layouts before compiling them to XML. |

## 📈 Coding Trends & Philosophy

1.  **Native over Web:** Moving away from heavy Electron apps to lightweight, compiled binaries that respect system resources.
2.  **Adaptive Layouts:** Using Libadwaita components so the app looks and works perfectly on both mobile (Handhelds/Phones) and large 4K monitors.
3.  **Concurrency:** Leveraging Rust’s ownership model to poll system data (CPU/RAM) on background threads without crashing or freezing the user interface.
4.  **Hardware Acceleration:** GTK4 handles all rendering via the GPU, ensuring smooth 60+ FPS graph animations.

## 📦 Key Dependencies

* **`sysinfo` (v0.33):** The primary engine for gathering cross-platform system metrics like process lists, CPU usage, and memory.
* **`zbus` & `zbus_systemd`:** A modern D-Bus implementation used to talk to `systemd` and other system services asynchronously.
* **`tokio`:** The asynchronous runtime that manages background tasks and timers without blocking the UI main loop.
* **`cairo`:** The vector graphics library used for custom-drawing the real-time performance history graphs.

## 🏗 Why this Stack?

* **Safety:** Zero "Segmentation Faults" or memory leaks.
* **Speed:** Near-instant startup times and minimal idle CPU usage.
* **Design:** Follows the GNOME HIG (Human Interface Guidelines) for a professional, "first-party" feel.
