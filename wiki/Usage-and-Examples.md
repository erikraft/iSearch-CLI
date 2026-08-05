# Usage & Examples

This section details how to navigate the interactive terminal browser, customize settings, use keyboard shortcuts, and configure custom options.

---

## 🚀 Running the CLI

Launch the interactive prompt from your console:
```bash
isearch
```

You can also run subcommands directly from the host console:
```bash
isearch browse
isearch version
isearch version --check
isearch self-update
isearch donate
```

---

## ⌨️ Interactive TUI Keyboard Shortcuts

### General Browser Controls
* **`Esc` or `Q`:** Exits the active panel, download manager, or shuts down the application.
* **`L`:** Focuses the active URL Bar to accept direct entries or query strings.
* **`R`:** Reloads the currently active tab page.
* **`E`:** Swaps rendering backends (toggles between **Native** and **Chromium** modes).
* **`T`:** Generates a new browsing tab.
* **`Tab`:** Switches focus to the next sequential tab.
* **`W`:** Closes the focused active tab (or rotates the 3D Wireframe mesh model).
* **`H`:** Displays/hides the global keyboard help screen.
* **`K`:** Cycles through color themes (Dracula, Nord, Ocean, Monokai, Light, Default).
* **`P`:** Downloads the active file or HTML page.
* **`Up / Down Arrow Keys`:** Scrolls the main layout viewport up or down.

---

### 🕵️‍♂️ Private Browsing / Anonymous Mode (`V`)

Toggle Private Browsing Mode anytime by pressing **`V`**. When active:
- A prominent purple-highlighted **🕵️‍♂️ [PRIVATE MODE]** indicator is shown on screen.
- Persistent database SQLite history logs are fully disabled.
- Cookies and browser caches are sandboxed in local temporary profiles using isolated `--incognito` arguments for Chromium.
- Download lists are stored strictly in-memory (RAM) and are deleted on exit.

---

### 🗄️ SQLite History Manager Panel (`Y`)

Toggle the History Manager panel by pressing **`Y`**.
* **`Up / Down`:** Traverses historical records.
* **`Enter`:** Navigates directly to the highlighted address and closes the history pane.
* **`/` or `S`:** Activates full-text search across titles and URLs.
* **`F`:** Activates domain filtering (e.g., `google.com`).
* **`R`:** Toggles sorting order (visits count vs. timestamp).
* **`G`:** Toggles grouping layout (Date Grouping vs. Domain Grouping vs. Raw List).
* **`D` or `Delete`:** Removes the selected history row from the SQLite database.
* **`C` or `Backspace`:** Clears ALL records from the database.
* **`I`:** Imports backups from a custom JSON file (`history_import.json`).
* **`E` or `X`:** Exports current database items to a JSON file (`history_export.json`).

---

### 📁 Bookmarks / Favorites Panel (`O`)

Toggle the Bookmarks manager panel by pressing **`O`**.
* **`Up / Down`:** Highlights bookmark rows.
* **`Enter`:** Navigates the browser to the highlighted favorite URL and closes the pane.
* **`A`:** Adds the current webpage to bookmarks, prompting for a Title, URL, and Folder.
* **`D` or `Delete`:** Removes the selected bookmark.
* **`F`:** Filters visible items by Folder categories.
* **`I`:** Imports favorites from JSON.
* **`E` or `X`:** Exports favorites to JSON.

---

### 📦 ZIP Explorer & 3D Engine Control
* **ZIP Explorer:** Simply browse to any local `.zip` archive URL. You can scroll through the compressed structure and press **`Enter`** to extract and preview individual files on-the-fly.
* **3D Wireframe Viewer:** In mesh preview screens, use **`W`, `S`, `A`, `D`** (or Arrow Keys) to rotate mathematical mesh projection meshes in real time.

---

## 🛠️ Configuration Configuration (`config.toml`)

Customize local parameters by creating a `config.toml` file under your config path (`~/.config/isearch/config.toml` or alongside the executable):

```toml
[donation]
pix_key = "11925416678"
currency = "BRL"
default_values = [5, 10, 20, 50, 100]
```
