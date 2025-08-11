CREATE TYPE public.image_info_type AS
(
	id integer,
	avg_color character varying(6),
	variants image_variant[]
);