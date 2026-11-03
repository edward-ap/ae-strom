import { icons } from './vendor/taet-ui.js';

// App-owned SVGs: speaker, gear, note and close paths retain the old mini-radio
// MIT source shapes. Tighter viewBoxes remove canvas margins, not hit targets.
// All vectors use Tæt's public registry and inherit the button's currentColor.
const definitions = {
  "radio-speaker": {
    "viewBox": "2 2 20 20",
    "nodes": [
      [
        "path",
        {
          "fill": "currentColor",
          "d": "M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"
        }
      ]
    ]
  },
  "radio-muted": {
    "viewBox": "4 2 16 20",
    "nodes": [
      [
        "path",
        {
          "fill": "currentColor",
          "d": "M7 9v6h4l5 5V4l-5 5H7z"
        }
      ]
    ]
  },
  "radio-settings": {
    "viewBox": "2 2 16 16",
    "nodes": [
      [
        "path",
        {
          "fill": "currentColor",
          "d": "M15.95 10.78c.03-.25.05-.51.05-.78s-.02-.53-.06-.78l1.69-1.32c.15-.12.19-.34.1-.51l-1.6-2.77c-.1-.18-.31-.24-.49-.18l-1.99.8c-.42-.32-.86-.58-1.35-.78L12 2.34c-.03-.2-.2-.34-.4-.34H8.4c-.2 0-.36.14-.39.34l-.3 2.12c-.49.2-.94.47-1.35.78l-1.99-.8c-.18-.07-.39 0-.49.18l-1.6 2.77c-.1.18-.06.39.1.51l1.69 1.32c-.04.25-.07.52-.07.78s.02.53.06.78L2.37 12.1c-.15.12-.19.34-.1.51l1.6 2.77c.1.18.31.24.49.18l1.99-.8c.42.32.86.58 1.35.78l.3 2.12c.04.2.2.34.4.34h3.2c.2 0 .37-.14.39-.34l.3-2.12c.49-.2.94-.47 1.35-.78l1.99.8c.18.07.39 0 .49-.18l1.6-2.77c.1-.18.06-.39-.1-.51l-1.67-1.32zM10 13c-1.65 0-3-1.35-3-3s1.35-3 3-3 3 1.35 3 3-1.35 3-3 3z"
        }
      ]
    ]
  },
  "radio-music": {
    "viewBox": "4 2 16 20",
    "nodes": [
      [
        "path",
        {
          "fill": "currentColor",
          "d": "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
        }
      ]
    ]
  },
  "radio-close": {
    "viewBox": "4 4 16 16",
    "nodes": [
      [
        "path",
        {
          "fill": "currentColor",
          "d": "M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
        }
      ]
    ]
  },
  "radio-play": {
    "viewBox": "2 2 20 20",
    "nodes": [
      [
        "path",
        {
          "fill": "currentColor",
          "d": "M7 3 22 12 7 21Z"
        }
      ]
    ]
  },
  "radio-stop": {
    "viewBox": "2 2 20 20",
    "nodes": [
      [
        "rect",
        {
          "fill": "currentColor",
          "x": 5,
          "y": 5,
          "width": 14,
          "height": 14,
          "rx": 0.5
        }
      ]
    ]
  },
  "radio-collapse": {
    "viewBox": "3 3 18 18",
    "nodes": [
      [
        "path",
        {
          "d": "M12 5 5 12 12 19",
          "fill": "none",
          "stroke": "currentColor",
          "stroke-width": 2,
          "stroke-linecap": "round",
          "stroke-linejoin": "round"
        }
      ],
      [
        "path",
        {
          "d": "M19 5 12 12 19 19",
          "fill": "none",
          "stroke": "currentColor",
          "stroke-width": 2,
          "stroke-linecap": "round",
          "stroke-linejoin": "round"
        }
      ]
    ]
  },
  "radio-expand": {
    "viewBox": "3 3 18 18",
    "nodes": [
      [
        "path",
        {
          "d": "M5 5 12 12 5 19",
          "fill": "none",
          "stroke": "currentColor",
          "stroke-width": 2,
          "stroke-linecap": "round",
          "stroke-linejoin": "round"
        }
      ],
      [
        "path",
        {
          "d": "M12 5 19 12 12 19",
          "fill": "none",
          "stroke": "currentColor",
          "stroke-width": 2,
          "stroke-linecap": "round",
          "stroke-linejoin": "round"
        }
      ]
    ]
  }
};

export function registerRadioIcons() {
  for (const [name, definition] of Object.entries(definitions)) {
    if (!icons.has(name)) icons.register(name, definition);
  }
}
