# 🛠️ **Check Node Packages (cnp)**

A utility tool written in **Rust** to check for unused node packages in your project. It helps you identify and clean up dependencies that are no longer needed.

## ✨ Features

- Scans files for dependency usage.
- Supports `.cnpignore` for excluding dependencies.
- Interactive mode for reviewing deletions.
- Clear, tabular output with progress feedback.

## 📚 Usage

```bash
cnp           # Scan and report unused dependencies
cnp --dry-run # Preview without changes
cnp --clean   # Interactively remove unused dependencies
```

## Configuration

- **`.cnpignore`**: List dependencies to exclude (one per line, `#` for comments).

```text
react-dom
eslint
```

## 🧑‍💻 Contributing

Feel free to open issues or pull requests to help improve the tool! Contributions are always welcome 🌟.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
