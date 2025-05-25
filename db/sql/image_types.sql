CREATE TYPE image_variant AS
(
	width integer,
	height integer,
	variant integer
);

DROP TYPE IF EXISTS public.image_info_type;

CREATE TYPE public.image_info_type AS
(
	id integer,
	avg_color character varying(6),
	variants image_variant[]
);