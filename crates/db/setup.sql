CREATE USER sjf_user NOCREATEDB NOCREATEUSER
CREATE SCHEMA IF NOT EXISTS sjf AUTHORIZATION 


ALTER TABLE product_categories_hierarchy DROP CONSTRAINT decendant_pkey;
ALTER TABLE product_categories_hierarchy ADD CONSTRAINT ancestor_decendant_pkey PRIMARY KEY(ancestor,decendant);


ALTER TABLE product_categories_hierarchy DROP CONSTRAINT product_categories_hierarchy_pkey;