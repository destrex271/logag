import type { Plugin } from "@opencode-ai/plugin";

const sessions = new Map<string, {
  userInput: string;
  lastUserMessageID: string;
  assistantOutput: string;
}>();

export default (async () => {
  return {
    "chat.message": async (input, output) => {
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

  fetch("http://localhost:8000/record", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ sessionId: sessionID, userInput, agentOutput: cleanAgentOutput }),
  }).catch((err) => {
    console.error("Translation Server Offline:", err.message);
  });
}
