# Displays Controller

### A tool that makes it easy to **view** and **control** all displays connected to your computer

#### Supported Browsers 

[![Google Chrome](https://img.shields.io/badge/Google%20Chrome-4285F4?logo=GoogleChrome&logoColor=white)](#)
[![Edge](https://custom-icon-badges.demolab.com/badge/Microsoft%20Edge-2771D8?logo=edge-white&logoColor=white)](#)

#### Supported Systems

[![Windows](https://custom-icon-badges.demolab.com/badge/Windows-0078D6?logo=windows11&logoColor=white)](#)

## Usage

### Connect your displays

![1.gif](front/src/assets/1.gif)

### Maximize and minimize previews

![2.gif](front/src/assets/2.gif)

### Click to control _(p.s. Display is behind my back)_

![3.gif](front/src/assets/3.gif)

## Installation

**Requirements**
- [Rust](https://rustup.rs/) — for backend
- [Node.js (LTS)](https://nodejs.org/) — for frontend

```bash
git clone https://github.com/arkadii888/DisplaysController.git
cd DisplaysController/front

# Allow npm scripts to run in PowerShell (Windows only)
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned -Force

# Install frontend dependencies
npm install

# Run backend (starts both backend and frontend)
cd ../back
cargo run
```

![console.png](front/src/assets/console.png)


### After setup is complete, you can use Displays Controller anytime by simply running:

```bash
cd DisplaysController/back
cargo run
```
