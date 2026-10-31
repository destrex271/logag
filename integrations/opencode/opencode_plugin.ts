import type { Plugin } from "@opencode-ai/plugin";
import * as fs from "fs";
import * as path from "path";

const LOGAG_CONFIG_FILE = "logag_config.toml";
const LOGAG_RECORD_URL = "http://localhost:8000/record";
const MISSING_PROJECT_ID = "__MISSING_PROJECT_ID__";

const sessions = new Map<string, {
  userInput: string;
  lastUserMessageID: string;
  assistantOutput: string;
}>();

function getCwd(): string {
  try {
    return process.cwd();
  } catch {
    return ".";
  }
}

function parseTomlProjectId(content: string): string | null {
  const regex1 = /project[-_]id\s*=\s*"([^"]+)"/i;
  const match1 = content.match(regex1);
  if (match1) return match1[1];
  const regex2 = /project[-_]id\s*=\s*([^\s\n#]+)/i;
  const match2 = content.match(regex2);
  if (match2) return match2[1].replace(/['"]/g, "");
  return null;
}

function projectIdFromDir(dir: string): string | null {
  const configPath = path.join(dir, LOGAG_CONFIG_FILE);
  if (!fs.existsSync(configPath)) {
    return null;
  }
  try {
    const content = fs.readFileSync(configPath, "utf-8");
    return parseTomlProjectId(content);
  } catch {
    return null;
  }
}

function getProjectIdMessage(): string {
  return "A project name is required to work with logag -> ask the user for a project name. Once you have the project name, create a logag_config.toml file in the current working directory with `project-id = \"<uuid>\"` by using the create_project functionality via MCP if available, or ask the user to provide the project ID.";
}

function appendTextPart(output: any, text: string) {
  const message = output?.message;
  if (!Array.isArray(output?.parts) || !message?.id || !message?.sessionID) {
    return;
  }
  output.parts.push({
    id: `prt_${Date.now().toString(36)}${Math.random().toString(36).slice(2, 10)}`,
    sessionID: message.sessionID,
    messageID: message.id,
    type: "text",
    text,
  });
}

export default (async (input) => {
  const projectDirs = [input?.directory, input?.worktree, getCwd()].filter(
    (dir, index, all): dir is string => Boolean(dir) && all.indexOf(dir) === index
  );

  function getProjectId(): string {
    for (const dir of projectDirs) {
      const projectId = projectIdFromDir(dir);
      if (projectId) {
        return projectId;
      }
    }
    return MISSING_PROJECT_ID;
  }

  function sendPair(sessionID: string, userInput: string, agentOutput: string, output?: any) {
    const cleanAgentOutput = agentOutput.startsWith(userInput)
      ? agentOutput.slice(userInput.length)
      : agentOutput;

    const projectId = getProjectId();
    if (projectId === MISSING_PROJECT_ID) {
      return;
    }

    fetch(LOGAG_RECORD_URL, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        sessionId: sessionID,
        userInput,
        agentOutput: cleanAgentOutput,
        projectLane: projectId,
      }),
    }).catch((err) => {
      const msg = `Translation Server Offline: ${err.message}. LogAg server may not be running at ${LOGAG_RECORD_URL}`;
      appendTextPart(output, `\n[LOGAG ERROR] ${msg}\n`);
    });
  }

  return {
    "chat.message": async (input, output) => {
      const projectId = getProjectId();
      if (projectId === MISSING_PROJECT_ID) {
        appendTextPart(output, getProjectIdMessage());
        return;
      }
      const sessionID = input.sessionID;
      const userText = output.parts
        .filter((p): p is any => p.type === "text")
        .map(p => p.text)
        .join("");

      const prev = sessions.get(sessionID);

      if (prev?.userInput && prev.assistantOutput) {
        sendPair(sessionID, prev.userInput, prev.assistantOutput, output);
      }

      sessions.set(sessionID, {
        userInput: userText,
        lastUserMessageID: input.messageID ?? "",
        assistantOutput: "",
      });
    },

    "event": async ({ event }) => {
      const projectId = getProjectId();
      if (projectId === MISSING_PROJECT_ID) {
        return;
      }
      if (event.type === "message.part.updated") {
        const props = event.properties as any;
        const part = props.part ?? props;
        if (part.type !== "text") return;
        const sessionID = props.sessionID ?? part.sessionID;
        const session = sessions.get(sessionID);
        if (session && part.messageID !== session.lastUserMessageID) {
          session.assistantOutput += part.text;
        }
      }
    },
  };
}) satisfies Plugin;
