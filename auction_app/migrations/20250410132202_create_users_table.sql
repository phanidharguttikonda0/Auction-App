-- Fix 1: CREATE TABLE not "create"
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(18) NOT NULL UNIQUE,
    mail_id VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(60) NOT NULL -- Make room for bcrypt hash
);

CREATE TABLE rooms (
    room_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_id INT REFERENCES users(id),
    accessibility VARCHAR(10) CHECK (accessibility IN ('public', 'private')),
    room_status VARCHAR(20) CHECK (room_status IN ('ongoing', 'waiting', 'completed'))
);

CREATE TABLE stats (
    id SERIAL PRIMARY KEY,
    matches INT,
    total_runs INT,
    average FLOAT,
    strike_rate FLOAT,
    fifties INT,
    hundreads INT,
    wickets INT,
    three_wickets INT,
    five_wickets INT
);

CREATE TABLE players (
    player_id SERIAL PRIMARY KEY,
    player_name TEXT NOT NULL,
    age INT NOT NULL,
    country VARCHAR(50) NOT NULL,
    role VARCHAR(20) NOT NULL,
    capped BOOLEAN NOT NULL,
    stats INT REFERENCES stats(id),
    pool VARCHAR(1) NOT NULL
);

CREATE TABLE participants (
    participant_id SERIAL PRIMARY KEY,
    team_selected VARCHAR(50) NOT NULL CHECK (
        team_selected IN (
            'MumbaiIndians',
            'ChennaiSuperKings', 
            'RoyalChallengesBengaluru',
            'SunrisersHyderabad',
            'KolkataKingKnightRiders',
            'PunjabKings',
            'DelhiCapitals',
            'RajastanRoyals',
            'LucknowSuperGaints',
            'GujaratTitans'
        )
    ),
    user_id INT REFERENCES users(id),
    room_id UUID REFERENCES rooms(room_id),
    UNIQUE(user_id, room_id),
    UNIQUE (room_id, team_selected)
);

CREATE TABLE players_unsold (
    player_id INT REFERENCES players(player_id),
    room_id UUID REFERENCES rooms(room_id),
    PRIMARY KEY (player_id, room_id)
);

CREATE TABLE bids (
    bid_id SERIAL PRIMARY KEY,
    room_id UUID REFERENCES rooms(room_id),
    player_id INT REFERENCES players(player_id),
    participant_id INT REFERENCES participants(participant_id),
    amount INT NOT NULL -- 50L 100L, 150L 400L(we will use L instead of cr)
);

