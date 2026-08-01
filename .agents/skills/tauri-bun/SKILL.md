---
name: tauri-bun
description: Guides the agent on executing Tauri CLI commands and development workflow using Bun as the package manager and runtime.
---

# Tauri with Bun Skill Instructions

This skill provides instructions on how to interact with and manage Tauri desktop applications (targeting Windows and Linux) that use **Bun** as their primary package manager and runtime.

## Project Identification
You should use this skill when:
1. The project has a `bun.lock` or `bun.lockb` file in the workspace root.
2. The project contains a `src-tauri` directory (indicating a Tauri project).
3. The `package.json` includes Tauri dependencies (like `@tauri-apps/api`, `@tauri-apps/cli`).

---

## Standard Tauri Bun Commands

When running or building the application, always prefer using **Bun** over `npm`, `yarn`, or `pnpm`.

| Action | Command | Description |
| :--- | :--- | :--- |
| **Install Dependencies** | `bun install` | Installs all project dependencies. |
| **Run Dev Server** | `bun tauri dev` | Starts the Tauri development window with hot-reload. |
| **Build for Production** | `bun tauri build` | Packages the application for production. |
| **Tauri System Info** | `bun tauri info` | Shows debug information about the system and dependencies. |
| **Add a Plugin** | `bun tauri add <plugin-name>` | Adds a Tauri plugin to the project. |

### Desktop Targets
This project focuses on Desktop platforms (**Windows** and **Linux**). Ensure any plugins or configurations added are compatible with these platforms.

*Note: If the `package.json` does not have a `"tauri": "tauri"` script, you can execute the command using `bunx tauri <command>` instead, or add `"tauri": "tauri"` to the `"scripts"` object in `package.json`.*

---

## Best Practices and Guidelines

1. **Do Not Mix Package Managers**: Never execute `npm install`, `yarn install`, or `pnpm install` in a Bun project. This creates multiple lockfiles and will cause inconsistency.
2. **Execute Commands via Cwd**: Always run commands from the workspace root where `package.json` is located.
3. **Verify Configuration**:
   - The Tauri configuration is located in [tauri.conf.json](file:///d:/Project/hikariel/src-tauri/tauri.conf.json).
   - Verify that the `beforeDevCommand` and `beforeBuildCommand` in [tauri.conf.json](file:///d:/Project/hikariel/src-tauri/tauri.conf.json) also use Bun (e.g., `bun run dev` and `bun run build`).
