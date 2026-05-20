CREATE TABLE IF NOT EXISTS LogEvent{
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    content TEXT,
    timestamp TIMESTAMP
    event_type VARCHAR(20) NOT NULL CHECK (event_type IN ('user_input', 'agent_output'))
}
