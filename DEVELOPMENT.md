# Internal Development Guide

> [!TIP]
> Looking for the **Contributing Guidelines**? Visit [CONTRIBUTING.md](./CONTRIBUTING.md).

---

## Branching Strategy

We follow a structured iteration cycle to ensure stability across releases.

### Pre-release Phase
Each stable development iteration is committed to a dedicated alpha branch:
`alpha-stable-#` (e.g., `alpha-stable-1`)

### Post-release Phase
Each stable iteration is committed to a dedicated beta branch:
`beta-stable-#`

---

## Commit Policy

All commit messages must be **clear, descriptive**, and **sufficiently detailed**. We follow the "Imperative" style:
* ✅ `Fix: resolve graph clipping on high-res displays`
* ❌ `Fixed the graphs`

---

## Restrictions
To maintain project integrity, contributors must **not**:
- [ ] Modify existing dependencies without justification and peer review.
- [ ] Label non-functional builds as "stable".
- [ ] Alter or rewrite history on previous stable branches.
- [ ] Deviate from the guidelines defined in [STACK.md](./STACK.md).

---

## Development Environment

| Requirement | Recommendation |
| :--- | :--- |
| **Operating System** | Debian 13 "Trixie" Live GNOME |
| **IDE** | Visual Studio Code (Latest) |
| **Tooling** | Rust Toolchain & Blueprint Compiler |

### Automated Tooling Setup

Run the following commands to provision your environment:

#### 1. Install System Dependencies
```bash
sudo apt update && sudo apt install -y \
  libgtk-4-dev \
  libadwaita-1-dev \
  libglib2.0-dev \
  libcairo2-dev \
  build-essential \
  python3-pip
  pip install blueprint-compiler --break-system-packages
```
```Bash
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh -s -- -y
source $HOME/.cargo/env
rustup update stable
```
