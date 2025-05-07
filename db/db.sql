drop table if exists seat;

drop table if exists room;

drop table if exists guest_token;

drop table if exists guest;

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
	unique (room_id, guest_id),
	unique (room_id, position)
) strict;
