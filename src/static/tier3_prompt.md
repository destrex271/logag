# Skill to generate tier 3 memory for Agents

## What is tier 3 memory?

Generate facts from a set of shared conversation logs. Based on conversation logs, the following artifacts should be generated for the agent's use:

 - Clean up 'rejected' turn data from the conversation history.
 - Summarize entire conversation set.
 - Generate multiple possible user query variations to increase match criteria.

## Where are tier 3 responses stored?

 - Summary is discarded and is only for agent's internal contemplation, this is generated in tier 2 after holistcly reviewing tier 3 and 4.
 - Remove question and response data for 'rejected' turn.
 - Populate agent questions and link them to answers for the increased surface area. For this the agent response across similar queries should be consolidted into a single answer.

## Agent Response format:

 - Response format includes:
 - ```{"to_remove":[{<user_prompt>: <agent_response>}], "to_add": {"agent_response": <>, "related_prompts": [...]}}```
