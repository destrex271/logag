## Using LogAg

 - For every user query which pertains to:
    - A task: Some action that you need to perform.
    - A question: User wants you to answer a question he has.

 - call `get_cached_agent_response` from the `logag-read` MCP tool to check if there is already cached knowledge around that prompt. If cached content exists and is relevant, use it to inform your response.

 - Do not use cache for trivial queries that you already have loaded in your short term memory. The LogAg cache should be treated as a second tier memory, to be referened if you find yourself considering the use of tools like grep or any search for content.

 - LogAg identifies the current project by the `project-id` (a UUID) in `logag_config.toml` at the project root. The recording plugin reads this file, and every LogAg MCP call that takes a `project_lane` (for example `get_cached_agent_response` and `add_event`) must receive that same UUID. Never pass the project name.

 - If `logag_config.toml` is missing, ask the user for a project name, call `create_project_lane` from the `logag-project` MCP server, and write the returned UUID to `logag_config.toml` in the project root as `project-id = "<uuid>"`.
