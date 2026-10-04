CREATE TABLE IF NOT EXISTS ProjectLane (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name VARCHAR(1000)
);

-- Seed lane for callers that have no lane to attribute. The id is fixed so every
-- installation agrees on the same default lane.
INSERT INTO ProjectLane (id, name)
VALUES ('019b76da-a800-7ab1-800d-efaddedead01', 'default')
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS LogEvent (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    content TEXT,
    timestamp TIMESTAMP,
    event_type VARCHAR(20) NOT NULL CHECK (event_type IN ('user_input', 'agent_output')),
    project_lane UUID NOT NULL REFERENCES ProjectLane(id)
);

CREATE TABLE IF NOT EXISTS CachedAgentResponse (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    log_event_id UUID REFERENCES LogEvent(id),
    user_query_id UUID REFERENCES LogEvent(id),
    project_lane UUID NOT NULL REFERENCES ProjectLane(id)
);

CREATE TABLE IF NOT EXISTS UserInputEmbedding (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_event_id UUID REFERENCES LogEvent(id),
    embedding vector(384),
    project_lane UUID NOT NULL REFERENCES ProjectLane(id)
);