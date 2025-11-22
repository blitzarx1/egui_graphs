# Running egui_graphs Natively on Windows

## Option 1: Use Web Version (Easiest - Already Working!)

The web version is running at **http://127.0.0.1:8080/**
- Full functionality including your new color picker feature
- No additional setup needed
- Works perfectly from WSL

## Option 2: Install Rust on Windows (For Native Windows App)

1. **Download Rust for Windows:**
   - Visit: https://rustup.rs/
   - Download and run `rustup-init.exe`
   - Follow the installation prompts (default options are fine)

2. **Restart your terminal** (close and reopen PowerShell)

3. **Run the demo:**
   ```powershell
   cd "C:\Users\smateorodriguez\OneDrive - Deloitte (O365D)\Documents\personal-projects\egui_graphs"
   cargo run -p egui_graphs --example demo
   ```

## Option 3: WSLg (Windows 11 Only)

If you have Windows 11, WSLg should work automatically. Try:
```bash
wsl
cd '/mnt/c/Users/smateorodriguez/OneDrive - Deloitte (O365D)/Documents/personal-projects/egui_graphs'
cargo run -p egui_graphs --example demo
```

## Your New Color Picker Feature

In the app (web or native):
1. Press **Tab** or click **◀** (bottom-right) to open sidebar
2. Go to **Style** section
3. Scroll to **Custom Colors**
4. Enable and pick colors for nodes and edges!

## Current Status

✅ Code compiled successfully
✅ Web version running at http://127.0.0.1:8080/
✅ Color picker feature implemented and working
❌ Native execution blocked by missing display server in WSL

**Recommendation:** Use the web version - it's fast, fully functional, and requires no additional setup!
