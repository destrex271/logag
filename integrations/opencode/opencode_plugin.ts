import type { Plugin } from "@opencode-ai/plugin";
import * as fs from "fs";
import * as path from "path";
import { execSync } from "child_process";

const LOGAG_CONFIG_FILE = "logag_config.toml";
const LOGAG_RECORD_URL = "http://localhost:8000/record";
const LOGAG_CREATE_PROJECT_URL = "http://localhost:8000/create_project";

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

function getOrCreateProjectId(): string {
  if (cachedProjectId) {
    return cachedProjectId;
  }

  const cwd = getCwd();
  const configPath = path.join(cwd, LOGAG_CONFIG_FILE);

  // Check if config exists
  if (fs.existsSync(configPath)) {
    try {
      const content = fs.readFileSync(configPath, "utf-8");
      const projectId = parseTomlProjectId(content);
      if (projectId) {
        cachedProjectId = projectId;
        return cachedProjectId;
      }
    } catch (e) {
      // Fall through to create
    }
  }

  // Create project
  const projectName = path.basename(cwd) || "project";
  try {
    const cmd = `curl -s -X POST "${LOGAG_CREATE_PROJECT_URL}" -H "Content-Type: application/json" -d '{"projectName":"${projectName.replace(/"/g, '\\"')}","timestamp":"${Date.now().toString()}"}'`;
    try {
      const res = execSync(cmd, { timeout: 5000, stdio: ['ignore', 'pipe', 'ignore'] }).toString();
      const id = res.trim().replace(/^"/, '').replace(/"$/, '');
      if (id && id.length > 0 && !id.includes('<')) {
        cachedProjectId = id;
        try {
          const configContent = `project-id = "${id}"\n`;
          fs.writeFileSync(configPath, configContent);
        } catch (e) {
          console.error("Failed to write logag_config.toml:", e);
        }
        return cachedProjectId;
      }
    } catch (e) {
      // ignore
    }
  } catch (e) {
    console.error("Error in getOrCreateProjectId:", e);
  }

  // Try reading again
  if (fs.existsSync(configPath)) {
    try {
      const content = fs.readFileSync(configPath, "utf-8");
      const projectId = parseTomlProjectId(content);
      if (projectId) {
        cachedProjectId = projectId;
        return cachedProjectId;
      }
    } catch (e) {
    }
  }

  return cachedProjectId || "";
}

export default (async () => {
  // Initialize project on load
  getOrCreateProjectId();
  
  return {
    "chat.message": async (input, output) => {
      // Call method before every interaction
      getOrCreateProjectId();
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
      getOrCreateProjectId();
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

  const projectId = getOrCreateProjectId();

  fetch(LOGAG_RECORD_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ 
      sessionId: sessionID, 
      userInput, 
      agentOutput: cleanAgentOutput,
      projectLane: projectId,
      projectId: projectId,
      project_id: projectId,
    }),
  }).catch((err) => {
    console.error("Translation Server Offline:", err.message);
  });
}
