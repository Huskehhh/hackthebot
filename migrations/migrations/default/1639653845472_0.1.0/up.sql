
SET check_function_bodies = false;
CREATE SCHEMA htb;
CREATE TABLE htb.challenges (
    id integer NOT NULL,
    htb_id integer NOT NULL,
    name text NOT NULL,
    difficulty text NOT NULL,
    points integer NOT NULL,
    release_date text NOT NULL,
    category integer NOT NULL,
    machine_avatar text
);
CREATE SEQUENCE htb.challenges_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE htb.challenges_id_seq OWNED BY htb.challenges.id;
CREATE TABLE htb.solves (
    id integer NOT NULL,
    user_id integer NOT NULL,
    user_name text NOT NULL,
    challenge_id integer NOT NULL,
    solve_type text NOT NULL
);
CREATE SEQUENCE htb.solves_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
ALTER SEQUENCE htb.solves_id_seq OWNED BY htb.solves.id;
ALTER TABLE ONLY htb.challenges ALTER COLUMN id SET DEFAULT nextval('htb.challenges_id_seq'::regclass);
ALTER TABLE ONLY htb.solves ALTER COLUMN id SET DEFAULT nextval('htb.solves_id_seq'::regclass);
ALTER TABLE ONLY htb.challenges
    ADD CONSTRAINT challenges_htb_id_key UNIQUE (htb_id);
ALTER TABLE ONLY htb.challenges
    ADD CONSTRAINT challenges_pkey PRIMARY KEY (id);
ALTER TABLE ONLY htb.solves
    ADD CONSTRAINT solves_pkey PRIMARY KEY (id);
ALTER TABLE ONLY htb.solves
    ADD CONSTRAINT solves_challenge_id_fkey FOREIGN KEY (challenge_id) REFERENCES htb.challenges(htb_id) ON UPDATE RESTRICT ON DELETE RESTRICT;

SET check_function_bodies = false;
