-- Create subjects table
CREATE TABLE subjects (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create schemas table with global versioning
CREATE TABLE schemas (
    id SERIAL PRIMARY KEY,
    subject_id INT NOT NULL REFERENCES subjects(id) ON DELETE CASCADE,
    version INT NOT NULL,
    schema_text TEXT NOT NULL,
    schema_type VARCHAR(20) NOT NULL,
    "references" TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(subject_id, version)
);

-- Create indices for performance
CREATE INDEX idx_schemas_id ON schemas(id);
CREATE INDEX idx_schemas_subject_id ON schemas(subject_id);
CREATE INDEX idx_schemas_subject_version ON schemas(subject_id, version DESC);
CREATE INDEX idx_subjects_name ON subjects(name);
