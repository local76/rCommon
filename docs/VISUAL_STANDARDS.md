# local76 Ecosystem: Visual Branding & Icon Standards

This document establishes the design principles, visual guidelines, and technical specifications for icons in the `local76` suite of applications (`rFetch`, `rIdle`, `rMonitor`, `rWifi`, `rStartup`, `rTemplate`). 

By defining a unified visual identity, we ensure that every utility feels like a first-class, cohesive member of the same family while remaining individually recognizable.

---

## 1. Design Philosophy & Visual Language

The `local76` applications are local-first, lightweight developer utilities and system monitors. The visual language must reflect this purpose: **precise, high-tech, and minimal**, without feeling corporate or bland.

We define a signature style: **High-Contrast Monogram** (inspired by minimalist, clean, highly readable brand identities).

```
┌─────────────────────────────────────────┐
│               SQUIRCLE CONTAINER        │
│  ┌───────────────────────────────────┐  │
│  │     THIN BLACK OUTLINE BORDER     │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │   BOLD BLACK MONOGRAM       │  │  │
│  │  │   (e.g., rF)                │  │  │
│  │  └─────────────────────────────┘  │  │
│  │     SOLID WHITE BACKGROUND        │  │
│  └───────────────────────────────────┘  │
│               SOLID BLACK BASE          │
└─────────────────────────────────────────┘
```

### Key Design Pillars:
* **Solid White Squircle Base**: Icons use a solid white rounded square container as their base, providing maximum contrast and readability at all sizes.
* **Bold Black Monogram**: The foreground utilizes a clean, modern, bold black sans-serif monogram (representing the utility prefix, e.g. `rF` for `rFetch`) centered perfectly.
* **Thin Black Outline**: A crisp, thin black border details the perimeter of the squircle to maintain structure.
* **Ecosystem Uniformity**: Rather than using varied glowing accent colors, the core application icons are uniformly styled in high-contrast black-and-white. This establishes a clean, unified brand that looks professional on any wallpaper or taskbar.

---

## 2. Visual Monograms for local76 utilities

To make the icon set cohesive, all icons utilize a bold black monogram on a solid white squircle container:

| Application | Monogram | Suffix Meaning |
| :--- | :--- | :--- |
| **`rFetch`** | **`rF`** | System polling, hardware query, static gathering. |
| **`rMonitor`** | **`rM`** | Live metrics, resource utilization, telemetry. |
| **`rWifi`** | **`rW`** | Signal telemetry, connection state, wireless data. |
| **`rIdle`** | **`rI`** | Standby state, screensaver, screen locking. |
| **`rStartup`** | **`rS`** | Service launching, startup boot, initialization. |
| **`rTemplate`** | **`rT`** | Scaffold creation, skeleton template, boilerplate. |

---

## 3. Technical Specifications

To ensure the icons render crisp and sharp at all operating system scales (from desktop shortcuts down to taskbar icons), they must adhere to the following technical details:

### A. Grid & Stroke Scaling
* **Detail Grid**: Vector assets must be designed on a baseline **`24x24` grid** using a **`1.5px` stroke width**.
* **Vector Scaling**: When scaling the master icon canvas (e.g. `256x256`), the stroke weight must scale proportionally:
  * At `24x24` canvas: `1.5px` stroke
  * At `256x256` canvas: `16px` stroke
  * At `512x512` canvas: `32px` stroke
* **Line Caps & Joins**: All strokes must use `stroke-linecap: round` and `stroke-linejoin: round` to maintain smooth, soft endpoints.

### B. File Formats & Resolutions
* **`app_icon.png`**: High-resolution `512x512` or `256x256` 32-bit RGBA PNG with alpha transparency.
* **`app.ico`**: Multi-resolution Windows ICO container containing `256x256`, `48x48`, `32x32`, and `16x16` sizes.
* **Padding**: Leave a **15% padding margin** around the container bounds to allow glowing neon offsets and glares to fade out naturally without edge clipping.

---

## 4. Integration

Add references to these visual guidelines in project templates and packaging configurations to maintain brand consistency.
