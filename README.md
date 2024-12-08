# 🛠️ **Check Node Packages (cnp)**

A utility tool written in **Rust** to check for unused node packages in your project. It helps you identify and clean up dependencies that are no longer needed.
This project is **Work in Progress (WIP)** 🚧, so feel free to contribute!
Please note, that the current version (0.2.0) is not stable and don't work as expected yet.

## ✨ Features
- Check for unused node dependencies in your project.
- Option to **clean** and remove unused dependencies from your `package.json`.
- Scan files in your project to check which dependencies are used.
- Supports `js`, `ts`, `jsx`, and `tsx` file types.
- Skips directories like `node_modules`, `dist`, `build`, and others by default.

## 📚 Usage

1. **Clone this repo**:
    - `git clone https://github.com/trotelalexandre/cnp.git`
    - `cd cnp`

2. **Build the project**:
    - Make sure you have Rust installed. If not, install it. Then, build the project using:
      - `cargo build --release`

3. **Run the tool**:
    - To check for unused dependencies in your project, run:
      - `./target/release/cnp`

4. **Clean unused dependencies**:
    - To automatically remove unused dependencies from your `package.json`, run:
      - `./target/release/cnp --clean`

### Options

- `--clean` – Removes unused dependencies from your `package.json` file.

## 📝 TODO

- [ ] Implement better error handling 💥
- [ ] Add more file types for scanning 📝

## 🧑‍💻 Contributing

Feel free to open issues or pull requests to help improve the tool! Contributions are always welcome 🌟.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
