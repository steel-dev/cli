import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

const raw = execFileSync(
  "cargo",
  ["run", "--quiet", "--bin", "steel", "--", "--no-update-check", "--json", "describe", "--all"],
  { encoding: "utf8" },
);

const parsed = JSON.parse(raw);
const root = parsed.data;

function slug(command) {
  return command.toLowerCase().replaceAll(" ", "-");
}

function flatten(node, path = []) {
  const currentPath = node.command ? node.command.split(" ") : path;
  const own = currentPath.length > 1 ? [{ ...node, command: currentPath.join(" ") }] : [];
  const children = (node.subcommands || []).flatMap((child) =>
    flatten(
      {
        ...child,
        command: [...currentPath, child.name].join(" "),
      },
      currentPath,
    ),
  );
  return [...own, ...children];
}

function renderParameters(parameters = []) {
  if (!parameters.length) return "";
  const lines = ["### Parameters", ""];
  for (const parameter of parameters) {
    const names = [];
    if (parameter.short) names.push(`-${parameter.short}`);
    if (!parameter.positional) names.push(`--${parameter.name}`);
    if (parameter.positional) names.push(parameter.name);
    const required = parameter.required ? "required" : "optional";
    const description = parameter.description ? `: ${parameter.description}` : "";
    lines.push(`- \`${names.join(", ")}\` (${parameter.type}, ${required})${description}`);
  }
  lines.push("");
  return lines.join("\n");
}

function renderApiOperations(operations = []) {
  if (!operations.length) return "";
  const lines = ["### API Operations", ""];
  for (const operation of operations) {
    lines.push(
      `- \`${operation.operation_id}\` (${operation.status}): \`${operation.method} ${operation.path}\``,
    );
    lines.push(`  Example: \`${operation.example}\``);
    if (operation.streaming) {
      lines.push(`  Streaming: ${operation.streaming.transport} \`${operation.streaming.path}\``);
    }
  }
  lines.push("");
  return lines.join("\n");
}

const commands = flatten(root);
const lines = [
  "# Steel CLI Reference",
  "",
  "This file is generated from `steel describe --all` and API metadata.",
  "",
  "## Table of Contents",
  "",
  ...commands.map((command) => `- [${command.command}](#${slug(command.command)})`),
  "",
];

for (const command of commands) {
  lines.push(`## ${command.command}`, "");
  if (command.description) {
    lines.push(command.description, "");
  }
  lines.push("### Usage", "", "```bash", command.command, "```", "");
  if (command.aliases?.length) {
    lines.push(`Aliases: ${command.aliases.map((alias) => `\`${alias}\``).join(", ")}`, "");
  }
  const subcommands = command.subcommands || [];
  if (subcommands.length) {
    lines.push("### Subcommands", "");
    for (const subcommand of subcommands) {
      const description = subcommand.description ? `: ${subcommand.description}` : "";
      lines.push(`- \`${subcommand.name}\`${description}`);
    }
    lines.push("");
  }
  const parameters = renderParameters(command.parameters);
  if (parameters) lines.push(parameters);
  const apiOperations = renderApiOperations(command.api_operations);
  if (apiOperations) lines.push(apiOperations);
}

writeFileSync(resolve("docs/cli-reference.md"), `${lines.join("\n").trim()}\n`);
