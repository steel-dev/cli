#!/usr/bin/env node

// @steel-dev/cli npm shim — proxies to the native Steel binary.
// If the native binary isn't installed, prints installation instructions.

import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const BINARY_PATH = join(homedir(), ".steel", "bin", "steel");

if (existsSync(BINARY_PATH)) {
  // Proxy all args and stdio to the native binary
  const child = spawn(BINARY_PATH, process.argv.slice(2), {
    stdio: "inherit",
  });

  child.on("error", (err) => {
    console.error(`[steel] Failed to run native binary: ${err.message}`);
    process.exit(1);
  });

  child.on("exit", (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code ?? 1);
    }
  });
} else {
  console.error("Steel CLI native binary not found.");
  console.error("");
  console.error("The @steel-dev/cli npm package is now a migration shim.");
  console.error(
    "The actual CLI has moved to a native binary distributed via GitHub Releases."
  );
  console.error("");
  console.error("Install the native binary:");
  console.error("  curl -LsSf https://setup.steel.dev | sh");
  console.error("");
  console.error("Then add to your PATH:");
  console.error('  export PATH="$HOME/.steel/bin:$PATH"');
  console.error("");
  console.error(
    "Or download directly from: https://github.com/steel-dev/cli/releases"
  );
  process.exit(1);
}
