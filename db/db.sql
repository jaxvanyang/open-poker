create table guest (
	id integer primary key autoincrement check (id > 0),
	name text not null check (
		3 <= length (name)
		and length (name) <= 32
		and name not in ('system', 'server', 'client')
	),
	bankroll integer not null default 0
) strict;

create table guest_token (
	id integer references guest (id),
	token text not null unique check (length (token) = 64)
) strict;

create table room (
	id integer primary key autoincrement check (id > 0),
	sb integer not null default 0 check (
		0 <= sb
		and sb < 10
	)
) strict;

create table seat (
	room_id integer references room (id),
	position integer not null default 0 check (
		0 <= position
		and position < 10
	),
	guest_id integer references guest (id),
	ready integer not null default false check (ready in (true, false)),
	stack integer not null default 1000 check (stack >= 0),
	bet integer not null default 0 check (bet >= 0),
	fold integer not null default false check (fold in (true, false)),
	unique (room_id, guest_id),
	unique (room_id, position)
) strict;

create table game (
	id integer primary key autoincrement check (id > 0),
	room_id integer not null references room (id),
	round text not null default 'preflop' check (
		round in ('preflop', 'flop', 'turn', 'river', 'finish')
	),
	pot integer not null default 0 check (pot >= 0),
	position integer not null default 0 check (
		0 <= position
		and position < 10
	),
	raise_position integer not null default 0 check (
		0 <= raise_position
		and raise_position < 10
	),
	unique (id, room_id)
) strict;

create table hand (
	game_id integer references game (id),
	guest_id integer references guest (id),
	c1 text not null check (length (c1) = 2),
	c2 text not null check (length (c2) = 2),
	unique (game_id, guest_id)
) strict;

create table flop (
	game_id integer primary key references game (id),
	c1 text not null check (length (c1) = 2),
	c2 text not null check (length (c2) = 2),
	c3 text not null check (length (c3) = 2)
) strict;

create table turn (
	game_id integer primary key references game (id),
	card text not null check (length (card) = 2)
) strict;

create table river (
	game_id integer primary key references game (id),
	card text not null check (length (card) = 2)
) strict;

create table bet (
	game_id integer references game (id),
	guest_id integer references guest (id),
	chips integer not null check (chips >= 0),
	unique (game_id, guest_id)
) strict;

create table result (
	game_id integer references game (id),
	guest_id integer references guest (id),
	result integer not null,
	unique (game_id, guest_id)
) strict;
