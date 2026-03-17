#!/usr/bin/env node

// @steel-dev/cli postinstall — installs the native Steel binary via cargo-dist.
// This script runs automatically after `npm install -g @steel-dev/cli`.

import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const BINARY_DIR = join(homedir(), ".steel", "bin");
const BINARY_PATH = join(BINARY_DIR, "steel");
const INSTALLER_URL =
  "https://github.com/steel-dev/cli/releases/latest/download/steel-cli-installer.sh";

function main() {
  // Skip in CI unless explicitly requested
  if (process.env.CI && !process.env.STEEL_FORCE_INSTALL) {
    console.log("[steel] Skipping native binary install in CI.");
    console.log(
      "[steel] Set STEEL_FORCE_INSTALL=1 to override, or use: curl -LsSf https://steel.dev/install.sh | sh"
    );
    return;
  }

  // Already installed
  if (existsSync(BINARY_PATH)) {
    console.log(`[steel] Native binary already installed at ${BINARY_PATH}`);
    console.log("[steel] Run `steel update` to check for updates.");
    return;
  }

  console.log("[steel] Installing Steel CLI native binary...");
  console.log("");

  // Check for curl
  try {
    execSync("command -v curl", { stdio: "ignore" });
  } catch {
    console.error("[steel] Error: curl is required but not found.");
    console.error(
      "[steel] Install curl, then run: curl -LsSf https://steel.dev/install.sh | sh"
    );
    // Don't fail npm install — just warn
    return;
  }

  // Run the cargo-dist installer
  try {
    execSync(
      `curl --proto '=https' --tlsv1.2 -LsSf ${INSTALLER_URL} | sh`,
      { stdio: "inherit", shell: "/bin/sh" }
    );
    console.log("");
    console.log("[steel] Installation complete!");
    console.log(`[steel] Binary installed to ${BINARY_DIR}`);
    console.log(
      `[steel] Make sure ${BINARY_DIR} is in your PATH, then run: steel --help`
    );
  } catch (err) {
    console.error("");
    console.error("[steel] Automatic installation failed.");
    console.error("[steel] Install manually:");
    console.error(
      "  curl -LsSf https://steel.dev/install.sh | sh"
    );
    console.error("");
    console.error(
      "[steel] Or download from: https://github.com/steel-dev/cli/releases"
    );
    // Don't fail npm install
  }
}

main();
