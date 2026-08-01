# Nordic Breeze POS - Design Document

## 1. Product Concept
**Core Intent:** A Point of Sale system focusing on simplicity (Minimalist) and an airy feel (Airy) to reduce cognitive load during operation, utilizing a Scandinavian (Nordic) style characterized by cool white and blue tones.

---

## 2. Design System Tokens

### Color Palette
- **Primary:** `#5E81AC` (Frost Blue) - Used for primary buttons, active tabs, and highlights.
- **Background:** `#F4F7FA` (Icy Gray-Blue) - Main background of the system.
- **Surface:** `#FFFFFF` (Pure White) - For product cards, cart sidebar, and modals.
- **Text (Primary):** `#2E3440` (Deep Slate) - Main text and price/amount figures.
- **Muted:** `#D8DEE9` (Soft Gray-Blue) - Borders, secondary text, and disabled states.
- **Accent/Success:** `#A3BE8C` (Sage Green) - Successful payment status or in-stock badges.

### Typography
- **Font Family (Headings):** 'Noto Sans Thai', 'Outfit', sans-serif (600 Weight - Loopless / แบบไม่มีหัว)
- **Font Family (Body):** 'Noto Sans Thai', 'DM Sans', sans-serif (400-500 Weight - Loopless / แบบไม่มีหัว)
- **Scale:**
  - H1 (Amounts): 48px
  - H2 (Page Titles): 32px
  - Body: 16px
  - Small: 12px

### UI Properties
- **Border Radius:** 12px (md), 24px (lg)
- **Shadows:** Emphasize the use of a 1px border in `#D8DEE9` instead of shadows (Zero Shadows) to maintain a clean appearance.
- **Padding:** 24px - 32px (Generous whitespace)

### Don't

Avoid common design patterns frequently seen in AI-generated websites, such as:

* Full-screen purple, blue, or pink gradients
* Excessive glassmorphism
* Card components with identical border radius across all sections
* Decorative floating orbs or background blobs
* Glow effects around every element
* Hero sections consisting only of a centered heading and two buttons
* Grid layouts with 3 identical-looking cards
* Generic dashboard mockups
* Phrases like "Revolutionize your workflow"
* Vague, meaningless marketing copy
* Icons placed inside square boxes with gradient backgrounds
* Animations applied to every single element
* Overly large border radius
* Heavy shadows or excessive accent colors
* Adding elements just to fill space without a clear purpose

---
