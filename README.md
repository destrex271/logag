# LogAct implementation

This is an agentic harness to produce software with the following aims:
 - Agent Records its actions
 - Each loop uses understanding developed in previous runs
 - Every new agent spawn should have the identical + enhanced behavior of its predecessor. Eg: If agent crashes midway, new spawned agent 
   should resume as if it were a clone of that agent.




**THE WHAT**:  Logag is a tool which tracks user interaction and llm outputs and prepares a dictionary for the LLM. It also handles evolving context of the codebase/source data without forcing the LLM to read the entire codebase again.


**THE WHY**: LLMs/Agents waste a lot of tokens answering same user queries which usually have similar answers. This token usage can be better utilized across other functions like code generation or research from external sources like the web instead of re-reading an exisitng source.


**THE HOW**: Store a Cache of user queries to agent answers. Agent answers should be invalidated if there is an update in that specific part of the base static source, in this case a codebase.
