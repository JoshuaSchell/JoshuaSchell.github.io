---
title: "My Neovim Setup in 2025"
date: "Oct 20, 2025"
summary: "Plugins, structure, and keymaps that keep my terminal IDE fast."
---

I've been using Neovim for a few years now. Here's my current setup.

## Why Neovim?

- It's fast
- Highly customizable
- Runs in the terminal
- Great plugin ecosystem

## Key Plugins

| Plugin | Purpose |
|--------|---------|
| lazy.nvim | Package manager |
| telescope.nvim | Fuzzy finder |
| nvim-treesitter | Syntax highlighting |
| lsp-zero | LSP configuration |

## My Config Structure

```
~/.config/nvim/
├── init.lua
└── lua/
    └── config/
        ├── init.lua
        ├── keymaps.lua
        ├── lazy.lua
        └── set.lua
```

## Essential Keymaps

```lua
-- Leader key
vim.g.mapleader = " "

-- File explorer
vim.keymap.set("n", "<leader>pv", vim.cmd.Ex)

-- Move lines in visual mode
vim.keymap.set("v", "J", ":m '>+1<CR>gv=gv")
vim.keymap.set("v", "K", ":m '<-2<CR>gv=gv")

-- Keep cursor centered
vim.keymap.set("n", "<C-d>", "<C-d>zz")
vim.keymap.set("n", "<C-u>", "<C-u>zz")
```

## LSP Setup

The key to a good Neovim experience is LSP. I use `lsp-zero` to make setup easy:

```lua
local lsp = require('lsp-zero')

lsp.on_attach(function(client, bufnr)
  lsp.default_keymaps({buffer = bufnr})
end)

require('mason').setup({})
require('mason-lspconfig').setup({
  ensure_installed = {'ts_ls', 'rust_analyzer', 'lua_ls'},
  handlers = {
    lsp.default_setup,
  },
})
```

---

That's my setup! Feel free to steal any of it.
