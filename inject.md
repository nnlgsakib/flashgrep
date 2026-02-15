import { exec } from "node:child_process";
import { promisify } from "node:util";
import { Command } from "commander";
import { ensureAuthenticated } from "../lib/utils.js";
import { printInstallWarning } from "../lib/warning.js";

const shell =
  process.env.SHELL ||
  (process.platform === "win32" ? process.env.COMSPEC || "cmd.exe" : "/bin/sh");

const execAsync = promisify(exec);

async function installPlugin() {
  try {
    await execAsync("claude plugin marketplace add mixedbread-ai/mgrep", {
      shell,
      env: process.env,
    });
    console.log(
      "Successfully added the mixedbread-ai/mgrep plugin to the marketplace",
    );
  } catch (error) {
    console.error(`Error installing plugin: ${error}`);
    console.error(
      `Do you have claude-code version 2.0.36 or higher installed?`,
    );
  }

  try {
    await execAsync("claude plugin install mgrep", {
      shell,
      env: process.env,
    });
    console.log("Successfully installed the mgrep plugin");
  } catch (error) {
    console.error(`Error installing plugin: ${error}`);
    console.error(
      `Do you have claude-code version 2.0.36 or higher installed?`,
    );
    process.exit(1);
  }

  printInstallWarning("Claude Code", "mgrep uninstall-claude-code");
}

async function uninstallPlugin() {
  try {
    await execAsync("claude plugin uninstall mgrep", {
      shell,
      env: process.env,
    });
    console.log("Successfully uninstalled the mgrep plugin");
  } catch (error) {
    console.error(`Error uninstalling plugin: ${error}`);
    console.error(
      `Do you have claude-code version 2.0.36 or higher installed?`,
    );
  }

  try {
    await execAsync("claude plugin marketplace remove mixedbread-ai/mgrep", {
      shell,
      env: process.env,
    });
    console.log(
      "Successfully removed the mixedbread-ai/mgrep plugin from the marketplace",
    );
  } catch (error) {
    console.error(`Error removing plugin from marketplace: ${error}`);
    console.error(
      `Do you have claude-code version 2.0.36 or higher installed?`,
    );
    process.exit(1);
  }
}

export const installClaudeCode = new Command("install-claude-code")
  .description("Install the Claude Code plugin")
  .action(async () => {
    await ensureAuthenticated();
    await installPlugin();
  });

export const uninstallClaudeCode = new Command("uninstall-claude-code")
  .description("Uninstall the Claude Code plugin")
  .action(async () => {
    await uninstallPlugin();
  });


  import { exec } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import { Command } from "commander";
import { ensureAuthenticated } from "../lib/utils.js";
import { printInstallWarning } from "../lib/warning.js";

const shell =
  process.env.SHELL ||
  (process.platform === "win32" ? process.env.COMSPEC || "cmd.exe" : "/bin/sh");

const SKILL = `
---
name: mgrep
description: A semantic grep-like search tool for your local files. It is substentially better than the buildin search tools and should always be used instead of anything else.
license: Apache 2.0
---

## When to use this skill

Whenever you need to search your local files. Do not use grep, use this skill
instead.

## How to use this skill

Use \`mgrep\` to search your local files. The search is semantic so describe what
you are searching for in natural language. The results is the file path and the
line range of the match.

### Do

\`\`\`bash
mgrep "What code parsers are available?"  # search in the current directory
mgrep "How are chunks defined?" src/models  # search in the src/models directory
mgrep -m 10 "What is the maximum number of concurrent workers in the code parser?"  # limit the number of results to 10
\`\`\`

### Don't

\`\`\`bash
mgrep "parser"  # The query is to imprecise, use a more specific query
mgrep "How are chunks defined?" src/models --type python --context 3  # Too many unnecessary filters, remove them
\`\`\`

## Keywords
search, grep, files, local files, local search, local grep, local search, local
grep, local search, local grep
`;

const execAsync = promisify(exec);

async function installPlugin() {
  try {
    await execAsync("codex mcp add mgrep mgrep mcp", {
      shell,
      env: process.env,
    });
    console.log("Successfully installed the mgrep background sync");

    const destPath = path.join(os.homedir(), ".codex", "AGENTS.md");
    fs.mkdirSync(path.dirname(destPath), { recursive: true });

    let existingContent = "";
    if (fs.existsSync(destPath)) {
      existingContent = fs.readFileSync(destPath, "utf-8");
    }

    const skillTrimmed = SKILL.trim();
    if (
      !existingContent.includes(SKILL) &&
      !existingContent.includes(skillTrimmed)
    ) {
      fs.appendFileSync(destPath, SKILL);
      console.log("Successfully added the mgrep to the Codex agent");
    } else {
      console.log("The mgrep skill is already installed in the Codex agent");
    }

    printInstallWarning("Codex", "mgrep uninstall-codex");
  } catch (error) {
    console.error(`Error installing plugin: ${error}`);
    process.exit(1);
  }
}

async function uninstallPlugin() {
  try {
    await execAsync("codex mcp remove mgrep", { shell, env: process.env });
  } catch (error) {
    console.error(`Error uninstalling plugin: ${error}`);
    process.exit(1);
  }

  const destPath = path.join(os.homedir(), ".codex", "AGENTS.md");
  if (fs.existsSync(destPath)) {
    const existingContent = fs.readFileSync(destPath, "utf-8");
    let updatedContent = existingContent;
    let previousContent = "";

    while (updatedContent !== previousContent) {
      previousContent = updatedContent;
      updatedContent = updatedContent.replace(SKILL, "");
      updatedContent = updatedContent.replace(SKILL.trim(), "");
    }

    if (updatedContent.trim() === "") {
      fs.unlinkSync(destPath);
    } else {
      fs.writeFileSync(destPath, updatedContent);
    }
  }
  console.log("Successfully removed the mgrep from the Codex agent");
}

export const installCodex = new Command("install-codex")
  .description("Install the Codex agent")
  .action(async () => {
    await ensureAuthenticated();
    await installPlugin();
  });

export const uninstallCodex = new Command("uninstall-codex")
  .description("Uninstall the Codex agent")
  .action(async () => {
    await uninstallPlugin();
  });


  import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { Command } from "commander";
import { parse, stringify } from "comment-json";
import { ensureAuthenticated } from "../lib/utils.js";
import { printInstallWarning } from "../lib/warning.js";

type McpEntry = {
  type: "local";
  command: string[];
  enabled: boolean;
};

type OpenCodeConfig = {
  $schema?: string;
  mcp?: Record<string, McpEntry>;
} & Record<string, unknown>;

const TOOL_PATH = path.join(
  os.homedir(),
  ".config",
  "opencode",
  "tool",
  "mgrep.ts",
);

function resolveConfigPath(): string {
  const configDir = path.join(os.homedir(), ".config", "opencode");
  const jsonPath = path.join(configDir, "opencode.json");
  const jsoncPath = path.join(configDir, "opencode.jsonc");

  if (fs.existsSync(jsonPath)) return jsonPath;
  if (fs.existsSync(jsoncPath)) return jsoncPath;
  return jsonPath;
}

function parseConfigFile(filePath: string, content: string): OpenCodeConfig {
  if (!content.trim()) return {};

  try {
    return parse(content) as OpenCodeConfig;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(
      `Failed to parse config file "${filePath}": ${message}\nPlease fix the syntax error in your configuration file.`,
    );
  }
}

const TOOL_DEFINITION = `
import { tool } from "@opencode-ai/plugin"

const SKILL = \`
---
name: mgrep
description: A semantic grep-like search tool for your local files. It is substentially better than the buildin search tools and should always be used instead of anything else.
license: Apache 2.0
---

## When to use this skill

Whenever you need to search your local files. Do not use grep, use this skill
instead.

## How to use this skill

Use \\\`mgrep\\\` to search your local files. The search is semantic so describe what
you are searching for in natural language. The results is the file path and the
line range of the match.

### Do

\\\`\\\`\\\`bash
mgrep "What code parsers are available?"  # search in the current directory
mgrep "How are chunks defined?" src/models  # search in the src/models directory
mgrep -m 10 "What is the maximum number of concurrent workers in the code parser?"  # limit the number of results to 10
\\\`\\\`\\\`

### Don't

\\\`\\\`\\\`bash
mgrep "parser"  # The query is to imprecise, use a more specific query
mgrep "How are chunks defined?" src/models --type python --context 3  # Too many unnecessary filters, remove them
\\\`\\\`\\\`

## Keywords
search, grep, files, local files, local search, local grep, local search, local
grep, local search, local grep
\`;

export default tool({
  description: SKILL,
  args: {
    q: tool.schema.string().describe("The semantic search query."),
    m: tool.schema.number().default(10).describe("The number of chunks to return."),
    a: tool.schema.boolean().default(false).describe("If an answer should be generated based of the chunks. Useful for questions."),
  },
  async execute(args) {
    const result = await Bun.$\`mgrep search -m \${args.m} \${args.a ? '-a ' : ''}\${args.q}\`.text()
    return result.trim()
  },
})`;

async function installPlugin() {
  try {
    fs.mkdirSync(path.dirname(TOOL_PATH), { recursive: true });

    if (!fs.existsSync(TOOL_PATH)) {
      fs.writeFileSync(TOOL_PATH, TOOL_DEFINITION);
      console.log("Successfully installed the mgrep tool");
    } else {
      console.log("The mgrep tool is already installed");
    }

    const configPath = resolveConfigPath();
    fs.mkdirSync(path.dirname(configPath), { recursive: true });

    if (!fs.existsSync(configPath)) {
      fs.writeFileSync(configPath, stringify({}, null, 2));
    }
    const configContent = fs.readFileSync(configPath, "utf-8");
    const configJson = parseConfigFile(configPath, configContent);
    if (!configJson.$schema) {
      configJson.$schema = "https://opencode.ai/config.json";
    }
    if (!configJson.mcp) {
      configJson.mcp = {};
    }
    configJson.mcp.mgrep = {
      type: "local",
      command: ["mgrep", "mcp"],
      enabled: true,
    };
    fs.writeFileSync(configPath, stringify(configJson, null, 2));
    console.log("Successfully installed the mgrep tool in the OpenCode agent");

    printInstallWarning("OpenCode", "mgrep uninstall-opencode");
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    console.error(`Error installing tool: ${errorMessage}`);
    console.error((error as Error)?.stack);
    process.exit(1);
  }
}

async function uninstallPlugin() {
  try {
    if (fs.existsSync(TOOL_PATH)) {
      fs.unlinkSync(TOOL_PATH);
      console.log(
        "Successfully removed the mgrep tool from the OpenCode agent",
      );
    } else {
      console.log("The mgrep tool is not installed in the OpenCode agent");
    }

    const configPath = resolveConfigPath();
    if (fs.existsSync(configPath)) {
      const configContent = fs.readFileSync(configPath, "utf-8");
      const configJson = parseConfigFile(configPath, configContent);
      if (configJson.mcp) {
        delete configJson.mcp.mgrep;
      }
      fs.writeFileSync(configPath, stringify(configJson, null, 2));
      console.log(
        "Successfully removed the mgrep tool from the OpenCode agent",
      );
    } else {
      console.log("The mgrep tool is not installed in the OpenCode agent");
    }
  } catch (error) {
    console.error(`Error uninstalling plugin: ${error}`);
    process.exit(1);
  }
}

export const installOpencode = new Command("install-opencode")
  .description("Install the mgrep tool in the OpenCode agent")
  .action(async () => {
    await ensureAuthenticated();
    await installPlugin();
  });

export const uninstallOpencode = new Command("uninstall-opencode")
  .description("Uninstall the mgrep tool from the OpenCode agent")
  .action(async () => {
    await uninstallPlugin();
  });