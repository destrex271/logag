import type { Plugin } from "@opencode-ai/plugin";
import * as fs from "fs";
import * as path from "path";

const LOGAG_CONFIG_FILE = "logag_config.toml";
const LOGAG_RECORD_URL = "http://localhost:8000/record";

const sessions = new Map<string, {
  userInput: string;
  lastUserMessageID: string;
  assistantOutput: string;
}>();

let cachedProjectId: string | null = null;

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

function getProjectId(): string {
  if (cachedProjectId) {
    return cachedProjectId;
  }

  const cwd = getCwd();
  const configPath = path.join(cwd, LOGAG_CONFIG_FILE);

  if (fs.existsSync(configPath)) {
    try {
      const content = fs.readFileSync(configPath, "utf-8");
      const projectId = parseTomlProjectId(content);
      if (projectId) {
        cachedProjectId = projectId;
        return cachedProjectId;
      }
    } catch (e) {
      // ignore
    }
  }

  return "__MISSING_PROJECT_ID__";
}

function getProjectIdMessage(): string {
  return "A project name is required to work with logag -> ask the user for a project name. Once you have the project name, create a logag_config.toml file in the current working directory with `project-id = \"<uuid>\"` by using the create_project functionality via MCP if available, or ask the user to provide the project ID.";
}

export default (async () => {
  return {
    "chat.message": async (input, output) => {
      const projectId = getProjectId();
      if (projectId === "__MISSING_PROJECT_ID__") {
        const msg = getProjectIdMessage();
        console.error(msg);
        try {
          if (output.parts) {
            output.parts.push({ type: "text", text: msg } as any);
          }
        } catch (e) {
          // ignore
        }
        return;
      }
      const sessionID = input.sessionID;
      const userText = output.parts
        .filter((p): p is any => p.type === "text")
        .map(p => p.text)
        .join("");

      const prev = sessions.get(sessionID);

      if (prev?.userInput && prev.assistantOutput) {
        sendPair(sessionID, prev.userInput, prev.assistantOutput);
      }

      sessions.set(sessionID, {
        userInput: userText,
        lastUserMessageID: input.messageID ?? "",
        assistantOutput: "",
      });
    },

    "event": async ({ event }) => {
      const projectId = getProjectId();
      if (projectId === "__MISSING_PROJECT_ID__") {
        console.error(getProjectIdMessage());
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

function sendPair(sessionID: string, userInput: string, agentOutput: string) {
  const cleanAgentOutput = agentOutput.startsWith(userInput)
    ? agentOutput.slice(userInput.length)
    : agentOutput;

  const projectId = getProjectId();
  if (projectId === "__MISSING_PROJECT_ID__") {
    console.error(getProjectIdMessage());
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
    console.error("Translation Server Offline:", err.message);
  });
}
