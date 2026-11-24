// Renders the tray/menu-bar menu icons from the same lucide icon set the app
// uses, so native menus match the interface. Run: node scripts/render-menu-icons.mjs
//
// macOS draws custom menu item images 18 pt tall and does not tint them like
// template images, so each icon is rendered at 3x with system-like padding and
// in the menu label colour for both appearances (`<name>.png`, `<name>-dark.png`).
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { chromium } from "@playwright/test";
import { AppWindow, Pause, Play, Settings, Power, Square, SquareCheck } from "lucide-react";
import { mkdirSync } from "node:fs";

const out = new URL("../src-tauri/icons/menu/", import.meta.url);
mkdirSync(out, { recursive: true });
const icons = {
  open: AppWindow,
  pause: Pause,
  play: Play,
  settings: Settings,
  quit: Power,
  unchecked: Square,
  checked: SquareCheck,
};
const canvas = 54; // 18 pt at 3x
const glyph = 45; // ~15 pt of artwork, like SF Symbols at menu size
const themes = { "": "rgba(0,0,0,0.85)", "-dark": "rgba(255,255,255,0.85)" };
const browser = await chromium.launch();
const page = await browser.newPage({ deviceScaleFactor: 1 });
for (const [name, Icon] of Object.entries(icons)) {
  for (const [suffix, color] of Object.entries(themes)) {
    const svg = renderToStaticMarkup(
      React.createElement(Icon, { size: glyph, color, strokeWidth: 1.6 }),
    );
    await page.setContent(
      `<body style="margin:0;background:transparent"><div id="i" style="width:${canvas}px;height:${canvas}px;display:grid;place-items:center">${svg}</div></body>`,
    );
    await page.locator("#i").screenshot({
      path: new URL(`${name}${suffix}.png`, out).pathname,
      omitBackground: true,
    });
  }
}
await browser.close();
console.log(`Rendered ${Object.keys(icons).length * 2} menu icons.`);
