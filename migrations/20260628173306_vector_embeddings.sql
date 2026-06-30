CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE IF NOT EXISTS CachedAgentResponse (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    log_event_id UUID REFERENCES LogEvent(id),
    user_query_id UUID REFERENCES LogEvent(id)
);

CREATE TABLE IF NOT EXISTS UserInputEmbedding (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_event_id UUID REFERENCES LogEvent(id),
    embedding vector(384)
);
