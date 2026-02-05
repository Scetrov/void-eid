# Design System

## Typography

### Headers / Monospace

- **Font Family**: `Diskette Mono`, monospace
- **Usage**: Headings (`h1`, `h2`, `h3`), Code blocks, Buttons

### Body

- **Font Family**: `'Inter'`, system-ui, -apple-system, sans-serif
- **Usage**: Paragraphs, Interface text

## Color Palette

### Theme Usage Guidance

> [!TIP]
> **Always** use usage-based variables (e.g., `--bg-primary`, `--text-primary`) instead of hardcoded colors. These variables automatically switch values based on the active theme (Light/Dark).

| Variable           | Light Mode (Stone/Warm)    | Dark Mode (Red/Nebula)   | Usage Meaning               |
| ------------------ | -------------------------- | ------------------------ | --------------------------- |
| `--bg-primary`     | `#fafaf9` (Stone 50)       | `#1a0a0a` (Deep Dark)    | Main page background        |
| `--bg-secondary`   | `#e7e5e4` (Stone 200)      | `#2d1515` (Dark Red)     | Inputs, code blocks, panels |
| `--text-primary`   | `#1c1917` (Stone 900)      | `#fafae5` (Cream White)  | Main headings, body text    |
| `--text-secondary` | `#57534e` (Stone 600)      | `#c7b8b3` (Light Stone)  | Subtitles, meta-data        |
| `--card-bg`        | `rgba(250, 250, 229, 0.9)` | `rgba(45, 21, 21, 0.85)` | Card backgrounds            |
| `--border-color`   | `#a8a29e` (Stone 400)      | `#6b6b5e` (Stone 500)    | Borders, dividers           |

### Brand Colors (Universal)

| Variable         | Value     | Usage                       |
| ---------------- | --------- | --------------------------- |
| `--brand-orange` | `#FF7400` | Primary Actions, Highlights |
| `--cream-white`  | `#fafae5` | Light Text, Accents         |

## Components

- **Buttons**: `Diskette Mono` font, uppercase, strict square corners (`0px` radius).
- **Cards**: "Frontier" style with angular brackets, technical borders, and no box shadow.
- **Inputs**: Square corners, minimal, technical feel.
- **Global**: Border radius is explicitly disabled (`0px`) for a technical, industrial sci-fi look (EVE Frontier inspired).
