---
title: Documentation
description: How to contribute to the documentation.
---

* You learned something about `crate2nix` that isn't documented or described in a
bad way?
* You noticed some bad grammar?
* You spotted some overcomplicated wording?
* You want to spice up the layout?

🚀 Awesome. Contribute by improving the documentation! If you are not sure about
something, make a rough draft or explain what you want to do, submit a PR and ask
for feedback early.

## ⚡ Editing directly in GitHub

:::note
Only recommended for small changes.
:::

If you want to edit an already existing page, you can click on
the "Edit page" link at the bottom of the page. It will lead you to in-place
editing in GitHub.

This will eventually allow you to create a pull request with your changes.
Check for any "build errors" -- there is some linting that is run across
the markdown files and it is rather strict.

## 💻 Editing the documentation in a local copy

`crate2nix` uses Starlight and Astro to provide this GitHub page. All the docs
are in the docs folder of the source repo.

### 🥇 Recommended workflow

* **Enter a dev shell**: `nix develop` or `direnv allow` if you use [direnv](https://direnv.net/).
* **Enter docs directory**: `cd docs`
* **Install dependencies**: `npm install`
* **Preview your docs continuously**: `npm run dev`
* **Edit Loop**
  * **Edit**: Make your changes.
  * **Preview**: Preview your changes [locally](http://localhost:4321/crate2nix/).
  * **Build in nix**: `git add` your changes.
  * **Lint**: `pre-commit` run local lints.
* **Build in nix**: run `nix flake check`.
* **Create your pull request**

### 🧞 Using NPM Commands

All commands are run from the docs folder of the project, from a terminal:

| Command                   | Action                                           |
| :------------------------ | :----------------------------------------------- |
| `nix develop`             | Enter a dev shell if you haven't enabled [direnv](https://direnv.net/) |
| `npm install`             | Installs dependencies                            |
| `npm run dev`             | Starts local dev server at `localhost:4321`      |
| `npm run build`           | Build your production site to `./dist/`          |
| `npm run preview`         | Preview your build locally, before deploying     |
| `npm run astro ...`       | Run CLI commands like `astro add`, `astro check` |
| `npm run astro -- --help` | Get help using the Astro CLI                     |

### 🧞 Linting, checking, building

| Command                   | Action                                           |
| :------------------------ | :----------------------------------------------- |
| `nix develop`             | Enter a dev shell if you haven't enabled [direnv](https://direnv.net/) |
| `nix build .#docs`        | Building docs with nix                           |
| `nix flake check`         | Running build AND lints                          |
| `pre-commit`              | Running pre-commit lints without committing.     |

## 🚀 Docs Project Structure

Inside of your Astro + Starlight project, you'll see the following folders and files:

```text
.
├── public/
├── src/
│   ├── assets/
│   ├── content/
│   │   ├── docs/
│   │   └── config.ts
│   └── env.d.ts
├── astro.config.mjs
├── package.json
└── tsconfig.json
```

Starlight looks for `.md` or `.mdx` files in the `src/content/docs/` directory.
Each file is exposed as a route based on its file name.

Images can be added to `src/assets/` and embedded in Markdown with a relative link.

Static assets, like favicons, can be placed in the `public/` directory.

## 👀 Want to learn more about Starlight/Astro?

Check out [Starlight’s docs](https://starlight.astro.build/),
read [the Astro documentation](https://docs.astro.build),
or jump into the [Astro Discord server](https://astro.build/chat).

## 👀 Want to learn more about structuring documentation?

And help me with it 😀

* Read [about the Diátaxis framework](https://diataxis.fr/).
